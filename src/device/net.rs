//! WiFi and HTTPS, on a thread of their own so the game never waits: the
//! thread joins the network named in `cfg.toml` (as the pollution firmware
//! does), then makes the requests the game sends it one at a time, and hands
//! back each answer by the number `get` or `put` gave it. Without WiFi every
//! request just fails, and the game carries on with what's in flash.

use std::sync::mpsc::{Receiver, Sender, channel};
use std::time::Duration;

use embedded_svc::http::Method;
use embedded_svc::http::client::Client;
use embedded_svc::io::Write;
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::hal::delay::FreeRtos;
use esp_idf_svc::hal::modem::Modem;
use esp_idf_svc::http::client::{Configuration, EspHttpConnection};
use esp_idf_svc::nvs::EspDefaultNvsPartition;
use esp_idf_svc::sys::{esp, esp_wifi_set_ps, wifi_ps_type_t_WIFI_PS_NONE};
use esp_idf_svc::wifi::{AuthMethod, BlockingWifi, ClientConfiguration, Configuration as WifiConfiguration, EspWifi};
use log::{info, warn};

#[toml_cfg::toml_config]
pub struct Config {
    #[default("")]
    wifi_ssid: &'static str,
    #[default("")]
    wifi_psk: &'static str,
    /// Empty keeps arrests on this device only.
    #[default("https://carmen-api.jonathan-orlowski.dev")]
    api_url: &'static str,
}

/// The bureau's server, without a trailing slash; None if there isn't one.
pub fn server() -> Option<&'static str> {
    Some(CONFIG.api_url.trim_end_matches('/')).filter(|s| !s.is_empty())
}

const CONNECT_ATTEMPTS: u32 = 5;
const RETRY_DELAY_MS: u32 = 3000;
/// As the desktop game's requests.
const TIMEOUT: Duration = Duration::from_secs(8);
/// TLS needs a deep stack (the pollution firmware's main task has 16K for it).
const STACK: usize = 16 * 1024;

struct Job {
    id: u32,
    method: Method,
    url: String,
    body: Option<String>,
}

/// What came back for a request: the body of a 2xx answer, or why not.
pub type Answer = Result<String, String>;

pub struct Net {
    jobs: Sender<Job>,
    answers: Receiver<(u32, Answer)>,
    next: u32,
}

impl Net {
    pub fn start(modem: Modem<'static>, sysloop: EspSystemEventLoop, nvs: EspDefaultNvsPartition) -> anyhow::Result<Net> {
        let (jobs, job_rx) = channel::<Job>();
        let (answer_tx, answers) = channel();
        std::thread::Builder::new().stack_size(STACK).spawn(move || {
            let wifi = match join(modem, sysloop, nvs) {
                Ok(wifi) => Some(wifi),
                Err(e) => {
                    warn!("no WiFi, so no bureau: {e}");
                    None
                }
            };
            let mut wifi = wifi;
            for job in job_rx {
                let answer = match wifi.as_mut() {
                    None => Err("no WiFi".to_string()),
                    Some(w) => match ensure_connected(w) {
                        Err(e) => Err(format!("WiFi down: {e}")),
                        Ok(()) => request(&job).map_err(|e| e.to_string()),
                    },
                };
                if let Err(e) = &answer {
                    warn!("{:?} {}: {e}", job.method, job.url);
                }
                if answer_tx.send((job.id, answer)).is_err() {
                    return;
                }
            }
        })?;
        Ok(Net { jobs, answers, next: 0 })
    }

    pub fn get(&mut self, url: &str) -> u32 {
        self.send(Method::Get, url, None)
    }

    pub fn put(&mut self, url: &str, body: String) -> u32 {
        self.send(Method::Put, url, Some(body))
    }

    fn send(&mut self, method: Method, url: &str, body: Option<String>) -> u32 {
        self.next += 1;
        let _ = self.jobs.send(Job { id: self.next, method, url: url.to_string(), body });
        self.next
    }

    /// Answers that have come in since last asked.
    pub fn poll(&self) -> Vec<(u32, Answer)> {
        self.answers.try_iter().collect()
    }
}

fn join(modem: Modem<'static>, sysloop: EspSystemEventLoop, nvs: EspDefaultNvsPartition) -> anyhow::Result<BlockingWifi<EspWifi<'static>>> {
    if CONFIG.wifi_ssid.is_empty() {
        anyhow::bail!("no network in cfg.toml");
    }
    let mut wifi = BlockingWifi::wrap(EspWifi::new(modem, sysloop.clone(), Some(nvs))?, sysloop)?;
    wifi.set_configuration(&WifiConfiguration::Client(ClientConfiguration {
        ssid: CONFIG.wifi_ssid.try_into().map_err(|_| anyhow::anyhow!("network name too long"))?,
        password: CONFIG.wifi_psk.try_into().map_err(|_| anyhow::anyhow!("password too long"))?,
        auth_method: AuthMethod::WPA2Personal,
        ..Default::default()
    }))?;
    wifi.start()?;
    // Modem sleep drops the link when the access point changes channel; the
    // board is on USB power, so it isn't needed.
    esp!(unsafe { esp_wifi_set_ps(wifi_ps_type_t_WIFI_PS_NONE) })?;
    info!("WiFi started, joining '{}'", CONFIG.wifi_ssid);
    let mut attempt = 1;
    loop {
        match try_connect(&mut wifi) {
            Ok(()) => break,
            Err(e) if attempt < CONNECT_ATTEMPTS => {
                warn!("WiFi attempt {attempt}/{CONNECT_ATTEMPTS} failed: {e}; retrying");
                let _ = wifi.disconnect();
                FreeRtos::delay_ms(RETRY_DELAY_MS);
                attempt += 1;
            }
            Err(e) => return Err(e),
        }
    }
    info!("WiFi connected: {:?}", wifi.wifi().sta_netif().get_ip_info()?.ip);
    Ok(wifi)
}

/// Rejoins if the link has dropped (the driver doesn't on its own).
fn ensure_connected(wifi: &mut BlockingWifi<EspWifi<'static>>) -> anyhow::Result<()> {
    if wifi.is_connected()? && wifi.is_up()? {
        return Ok(());
    }
    let _ = wifi.disconnect();
    try_connect(wifi)
}

fn try_connect(wifi: &mut BlockingWifi<EspWifi<'static>>) -> anyhow::Result<()> {
    wifi.connect()?;
    wifi.wait_netif_up()?;
    Ok(())
}

fn request(job: &Job) -> anyhow::Result<String> {
    let connection = EspHttpConnection::new(&Configuration {
        timeout: Some(TIMEOUT),
        // ESP-IDF's bundled root certificates, so HTTPS works without pinning the server's.
        crt_bundle_attach: Some(esp_idf_svc::sys::esp_crt_bundle_attach),
        ..Default::default()
    })?;
    let mut client = Client::wrap(connection);
    let body = job.body.as_deref().unwrap_or("");
    let length = body.len().to_string();
    let headers = [("content-type", "application/json"), ("content-length", length.as_str())];
    let mut request = client.request(job.method, &job.url, if job.body.is_some() { &headers } else { &[] })?;
    if job.body.is_some() {
        request.write_all(body.as_bytes())?;
        request.flush()?;
    }
    let mut response = request.submit()?;
    let status = response.status();
    let mut out = Vec::new();
    let mut buf = [0u8; 512];
    loop {
        let n = response.read(&mut buf)?;
        if n == 0 {
            break;
        }
        out.extend_from_slice(&buf[..n]);
    }
    if !(200..300).contains(&status) {
        anyhow::bail!("the server answered {status}");
    }
    Ok(String::from_utf8_lossy(&out).into_owned())
}

/// `name` made safe for a URL path: letters and digits kept, everything else
/// (spaces, Ñ, accents) percent-encoded as UTF-8.
pub fn escape(name: &str) -> String {
    name.bytes()
        .map(|b| if b.is_ascii_alphanumeric() || b"-._~".contains(&b) { (b as char).to_string() } else { format!("%{b:02X}") })
        .collect()
}
