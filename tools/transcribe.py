#!/usr/bin/env python3
"""Turns the original game's tunes into notes the piezo can play: writes
src/device/tunes.rs. Run it from this repository with the folder of Carmen's
recordings (needs ffmpeg and numpy):

    python3 tools/transcribe.py ../carmen/assets/sounds

The recordings play a bass line under a melody, so the melody is followed:
the strongest pitch between FLOOR and CEILING, every HOP, rounded to a
semitone and smoothed, with notes shorter than SHORTEST dropped. Each tune is
then raised by whole octaves into the piezo's loud range (it peaks near
4 kHz and is faint below 1 kHz). The result is only the tune, not the sound:
effects (keys, footsteps, the printer) are written by hand in sound.rs.
"""

import subprocess
import sys
from pathlib import Path

import numpy as np

RATE = 22050
HOP = 0.01  # seconds between pitch readings
WINDOW = 2048  # samples per reading: long enough for 180 Hz
FLOOR, CEILING = 180.0, 1500.0  # where the melodies are, above the bass
QUIET = 0.08  # below this share of a tune's loudest moment, it's a rest
SHORTEST = 0.05  # seconds
LOUD_RANGE = (1000.0, 2400.0)  # where the tune's middle note should land

TUNES = [
    ("HENCHMAN_STRIDE", "henchman-stride"),
    ("HENCHMAN_SNEAK", "henchman-sneak"),
    ("HENCHMAN_POPUP", "henchman-popup"),
    ("DANGER_AXE", "danger-axe"),
    ("DANGER_KNIFE", "danger-knife"),
    ("DANGER_GUN", "danger-gun"),
    ("CHASE", "chase"),
    ("CAPTURED", "captured"),
    ("DISAPPOINT", "disappoint"),
    ("COMPUTING", "computing"),
    ("TRAVELLING", "travelling"),
]


def load(path):
    raw = subprocess.run(
        ["ffmpeg", "-v", "error", "-i", str(path), "-ac", "1", "-ar", str(RATE), "-f", "s16le", "-"],
        check=True,
        capture_output=True,
    ).stdout
    return np.frombuffer(raw, dtype=np.int16).astype(float) / 32768


def pitches(x):
    """(midi note or None, seconds) every HOP."""
    hop, win = int(RATE * HOP), np.hanning(WINDOW)
    freqs = np.fft.rfftfreq(WINDOW * 4, 1 / RATE)
    band = (freqs >= FLOOR) & (freqs <= CEILING)
    readings = []
    for i in range(0, max(len(x) - WINDOW, 0), hop):
        spectrum = np.abs(np.fft.rfft(x[i : i + WINDOW] * win, WINDOW * 4))
        energy = spectrum[band]
        k = int(energy.argmax())
        readings.append((freqs[band][k], energy[k]))
    if not readings:
        return []
    loudest = max(e for _, e in readings)
    notes = [round(69 + 12 * np.log2(f / 440)) if e >= QUIET * loudest else None for f, e in readings]
    # A median over five readings smooths out single-reading jumps to the bass or a harmonic.
    smooth = []
    for i in range(len(notes)):
        near = [n for n in notes[max(0, i - 2) : i + 3]]
        sounding = sorted(n for n in near if n is not None)
        smooth.append(sounding[len(sounding) // 2] if len(sounding) > len(near) // 2 else None)
    return smooth


def notes_of(readings):
    """Runs of the same note, as [midi or None, seconds], short ones folded into the one before."""
    runs = []
    for n in readings:
        if runs and runs[-1][0] == n:
            runs[-1][1] += HOP
        else:
            runs.append([n, HOP])
    merged = []
    for n, d in runs:
        if merged and (d < SHORTEST or merged[-1][0] == n):
            merged[-1][1] += d
        else:
            merged.append([n, d])
    return merged


def octaves_up(notes):
    sounding = sorted(n for n, _ in notes if n is not None)
    if not sounding:
        return 0
    middle = 440 * 2 ** ((sounding[len(sounding) // 2] - 69) / 12)
    k = 0
    while middle * 2**k < LOUD_RANGE[0]:
        k += 1
    while middle * 2**k > LOUD_RANGE[1]:
        k -= 1
    return k


def main():
    folder = Path(sys.argv[1])
    out = [
        "//! The original game's tunes as notes for the piezo, written by",
        "//! `tools/transcribe.py` from Carmen's recordings: (pitch in Hz, 0 for a",
        "//! rest; milliseconds). Each is raised by whole octaves into the piezo's",
        "//! loud range. Don't edit by hand: rerun the tool.",
        "",
    ]
    for name, file in TUNES:
        notes = notes_of(pitches(load(folder / f"{file}.ogg")))
        k = octaves_up(notes)
        pairs = [(0 if n is None else round(440 * 2 ** ((n + 12 * k - 69) / 12)), round(d * 1000)) for n, d in notes]
        while pairs and pairs[0][0] == 0:
            pairs.pop(0)
        total = sum(d for _, d in pairs)
        out.append(f"/// `{file}.ogg`: {len(pairs)} notes, {total} ms, raised {k} octave{'s' if k != 1 else ''}.")
        out.append(f"pub const {name}: &[(u16, u16)] = &[")
        out.extend(f"    ({hz}, {ms})," for hz, ms in pairs)
        out.append("];")
        out.append("")
    Path("src/device/tunes.rs").write_text("\n".join(out))
    print("wrote src/device/tunes.rs")


if __name__ == "__main__":
    main()
