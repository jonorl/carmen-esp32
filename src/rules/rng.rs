//! A tiny seedable random number generator (SplitMix64), so the same seed
//! replays the same case without pulling in the `rand` crate.

pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Self {
        Rng(seed)
    }

    pub fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// A number from 0 to n - 1.
    pub fn below(&mut self, n: usize) -> usize {
        assert!(n > 0, "below(0)");
        // Widening multiply keeps the bias negligible for the small n we use.
        ((self.next_u64() as u128 * n as u128) >> 64) as usize
    }

    pub fn choice<'a, T>(&mut self, items: &'a [T]) -> &'a T {
        &items[self.below(items.len())]
    }

    pub fn shuffle<T>(&mut self, items: &mut [T]) {
        for i in (1..items.len()).rev() {
            items.swap(i, self.below(i + 1));
        }
    }

    /// k distinct items, in random order.
    pub fn sample<T: Clone>(&mut self, items: &[T], k: usize) -> Vec<T> {
        let mut pool = items.to_vec();
        self.shuffle(&mut pool);
        pool.truncate(k);
        pool
    }
}
