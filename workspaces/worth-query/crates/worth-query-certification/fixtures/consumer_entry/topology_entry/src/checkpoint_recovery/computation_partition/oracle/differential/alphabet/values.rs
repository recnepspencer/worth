//! Seeded choices and order-sensitive floating-point values.
use super::*;

impl Lcg {
    pub(in super::super::super) fn below(&mut self, bound: usize) -> usize {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        usize::try_from((self.0 >> 33) % u64::try_from(bound).unwrap()).unwrap()
    }

    /// A float of either sign whose exponent lies in [-40, 40], so a sum of
    /// such floats depends on its order.
    pub(super) fn value(&mut self) -> f64 {
        let mantissa = 1.0 + f64::from(u32::try_from(self.below(1 << 20)).unwrap()) / 1_048_576.0;
        let exponent = i32::try_from(self.below(81)).unwrap() - 40;
        let sign = if self.below(2) == 0 { 1.0 } else { -1.0 };
        sign * mantissa * 2_f64.powi(exponent)
    }

    /// One of the seeded regions.
    pub(super) fn region(&mut self) -> u32 {
        u32::try_from(self.below(usize::try_from(REGIONS).unwrap())).unwrap()
    }

    pub(super) fn pick<T: Copy>(&mut self, from: &[T]) -> T {
        from[self.below(from.len())]
    }
}
