/// The reproducible input to a simulation's scheduling stream.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Seed(u64);

impl Seed {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn value(self) -> u64 {
        self.0
    }

    pub const fn rng(self) -> Rng {
        Rng(self.0)
    }
}

/// SplitMix64 with the same seed, draw order and modulo selection as the
/// persistent-naming court. Choices must be supplied in deterministic order.
#[derive(Clone, Debug)]
pub struct Rng(u64);

impl Rng {
    pub fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut mixed = self.0;
        mixed = (mixed ^ (mixed >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        mixed = (mixed ^ (mixed >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        mixed ^ (mixed >> 31)
    }

    /// Chooses an index below a positive bound, consuming one draw.
    ///
    /// # Panics
    ///
    /// Panics if `bound` is zero.
    pub fn below(&mut self, bound: usize) -> usize {
        assert!(bound > 0, "a scheduling choice needs an enabled candidate");
        (self.next_u64() % bound as u64) as usize
    }

    /// Selects one enabled candidate, consuming one draw.
    ///
    /// # Panics
    ///
    /// Panics if `items` is empty.
    pub fn pick<T: Copy>(&mut self, items: &[T]) -> T {
        items[self.below(items.len())]
    }
}
