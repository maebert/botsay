//! The random numbers behind every bot.
//!
//! A bot must look the same for a given seed on every platform and Rust
//! version, so this is hand-rolled rather than taken from a crate whose output
//! could change: FNV-1a hashes the seed, and SplitMix64 generates from there.

// ╭──────────────────────────────────────────────╮
// │ Changing anything in here changes every bot. │
// │ Don't, unless you mean to.                   │
// ╰──────────────────────────────────────────────╯
//     ╲
//      ╲
//       ▗▙▄▄▟▖
//       ▐▐██▌▌
//       ▐████▌
//        ▌  ▐
pub struct Rng(u64);

impl Rng {
    pub fn from_seed(seed: &str) -> Self {
        let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
        for byte in seed.bytes() {
            hash ^= u64::from(byte);
            hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
        }
        Rng(hash)
    }

    fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }

    /// The modulo bias doesn't matter for the tiny ranges used here.
    fn below(&mut self, n: usize) -> usize {
        (self.next_u64() % n as u64) as usize
    }

    /// A number in `lo..=hi`.
    pub fn range(&mut self, lo: i32, hi: i32) -> i32 {
        lo + self.below((hi - lo + 1) as usize) as i32
    }

    pub fn pick<T: Copy>(&mut self, items: &[T]) -> T {
        items[self.below(items.len())]
    }
}
