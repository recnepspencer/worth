//! Pure immutable meaning. This owner has no Query or wire-codec dependency.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Ratio {
    numerator: u32,
    denominator: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RatioDenial {
    ZeroDenominator,
    InternalInconsistency,
}

impl Ratio {
    pub fn new(numerator: u32, denominator: u32) -> Result<Self, RatioDenial> {
        if denominator == 0 {
            return Err(RatioDenial::ZeroDenominator);
        }
        let certificate = gcd_certificate(numerator, denominator);
        certify(numerator, denominator, certificate)?;
        let divisor = certificate.0;
        Ok(Self {
            numerator: numerator / divisor,
            denominator: denominator / divisor,
        })
    }
    pub const fn numerator(self) -> u32 {
        self.numerator
    }
    pub const fn denominator(self) -> u32 {
        self.denominator
    }
    pub fn render(self) -> String {
        format!("{}/{}", self.numerator, self.denominator)
    }
}

// Extended Euclid supplies a certificate, never the final authority alone.
// For u32 inputs, fewer than 48 divisions suffice; coefficients/products fit i128.
fn gcd_certificate(numerator: u32, denominator: u32) -> (u32, i128, i128) {
    let (mut a, mut b) = (numerator, denominator);
    let (mut x, mut next_x) = (1_i128, 0_i128);
    let (mut y, mut next_y) = (0_i128, 1_i128);
    while b != 0 {
        let quotient = a / b;
        (a, b) = (b, a % b);
        (x, next_x) = (next_x, x - i128::from(quotient) * next_x);
        (y, next_y) = (next_y, y - i128::from(quotient) * next_y);
    }
    (a, x, y)
}

// Independent checker: divisibility plus Bezout's identity proves greatestness,
// without repeating Euclid or trusting its division sequence.
fn certify(
    numerator: u32,
    denominator: u32,
    (divisor, x, y): (u32, i128, i128),
) -> Result<(), RatioDenial> {
    let combination = i128::from(numerator).checked_mul(x).and_then(|left| {
        i128::from(denominator)
            .checked_mul(y)
            .and_then(|right| left.checked_add(right))
    });
    if divisor == 0
        || numerator % divisor != 0
        || denominator % divisor != 0
        || combination != Some(i128::from(divisor))
    {
        return Err(RatioDenial::InternalInconsistency);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn normalized_meaning_and_terminal_rendering_are_owned_here() {
        assert_eq!(Ratio::new(2, 4), Ratio::new(1, 2));
        assert_eq!(Ratio::new(2, 4).unwrap().render(), "1/2");
        assert_eq!(Ratio::new(0, u32::MAX).unwrap().render(), "0/1");
        assert_eq!(Ratio::new(u32::MAX, u32::MAX).unwrap().render(), "1/1");
        assert_eq!(Ratio::new(1, 0), Err(RatioDenial::ZeroDenominator));
    }
    #[test]
    fn corrupted_certificate_is_refused_by_the_independent_checker() {
        for certificate in [(0, 0, 0), (1, 0, 0), (3, 1, 0), (2, i128::MAX, 0)] {
            assert_eq!(
                certify(2, 4, certificate),
                Err(RatioDenial::InternalInconsistency)
            );
        }
        assert_eq!(certify(2, 4, (2, 1, 0)), Ok(()));
    }
}
