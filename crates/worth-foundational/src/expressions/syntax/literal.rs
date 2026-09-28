//! Exact decimal text of an authored float literal.
//!
//! Float literals keep their exact decimal value until a typed context rounds
//! them once: Float64 and Float32 contexts round the decimal directly, and a
//! quantity literal rounds only after applying its exact unit scale.

/// `digits * 10^exponent`, normalized: `digits` has no leading or trailing
/// zeros, and zero is `("0", 0)`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct DecimalText {
    digits: Box<str>,
    exponent: i32,
}

impl DecimalText {
    /// Parses `d+[.d+][(e|E)[+-]d+]`. Returns `None` for malformed text or an
    /// exponent outside `i32`.
    pub(crate) fn parse(text: &str) -> Option<Self> {
        let (mantissa, exponent) = match text.find(['e', 'E']) {
            Some(at) => (&text[..at], parse_exponent(&text[at + 1..])?),
            None => (text, 0),
        };
        let (whole, fraction) = mantissa.split_once('.').unwrap_or((mantissa, ""));
        let is_digits = |part: &str| part.bytes().all(|byte| byte.is_ascii_digit());
        if whole.is_empty() || !is_digits(whole) || !is_digits(fraction) {
            return None;
        }
        if mantissa.contains('.') && fraction.is_empty() {
            return None;
        }
        let fraction_len = i64::try_from(fraction.len()).ok()?;
        let digits = format!("{whole}{fraction}");
        Self::normalized(&digits, exponent.checked_sub(fraction_len)?)
    }

    /// The exact decimal of the shortest round-trip rendering of `value`.
    pub(crate) fn from_f64(value: f64) -> Option<Self> {
        if !value.is_finite() || value.is_sign_negative() && value != 0.0 {
            return None;
        }
        Self::parse(&format!("{:e}", value.abs()))
    }

    fn normalized(digits: &str, exponent: i64) -> Option<Self> {
        let significant = digits.trim_start_matches('0');
        if significant.is_empty() {
            return Some(Self {
                digits: "0".into(),
                exponent: 0,
            });
        }
        let trimmed = significant.trim_end_matches('0');
        let removed = i64::try_from(significant.len() - trimmed.len()).ok()?;
        let exponent = i32::try_from(exponent.checked_add(removed)?).ok()?;
        Some(Self {
            digits: trimmed.into(),
            exponent,
        })
    }

    pub(crate) fn digits(&self) -> &str {
        &self.digits
    }

    pub(crate) fn exponent(&self) -> i32 {
        self.exponent
    }

    /// Correctly rounded (nearest, ties to even); `None` when not finite.
    pub(crate) fn to_f64(&self) -> Option<f64> {
        let value: f64 = self.scientific().parse().ok()?;
        value.is_finite().then_some(value + 0.0)
    }

    /// Correctly rounded directly from the decimal, never through Float64.
    pub(crate) fn to_f32(&self) -> Option<f32> {
        let value: f32 = self.scientific().parse().ok()?;
        value.is_finite().then_some(value + 0.0)
    }

    fn scientific(&self) -> String {
        format!("{}e{}", self.digits, self.exponent)
    }
}

fn parse_exponent(text: &str) -> Option<i64> {
    let (negative, digits) = match text.as_bytes().first()? {
        b'+' => (false, &text[1..]),
        b'-' => (true, &text[1..]),
        _ => (false, text),
    };
    if digits.is_empty() || !digits.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    let digits = digits.trim_start_matches('0');
    // Exponents past this bound overflow the normalized `i32` exponent anyway.
    if digits.len() > 12 {
        return None;
    }
    let magnitude: i64 = if digits.is_empty() {
        0
    } else {
        digits.parse().ok()?
    };
    Some(if negative { -magnitude } else { magnitude })
}

#[cfg(test)]
mod tests {
    use super::DecimalText;

    #[test]
    fn normalizes_equivalent_spellings() {
        let expected = DecimalText::parse("15e-1").unwrap();
        for text in ["1.5", "1.50", "0001.5", "0.15e1", "150e-2"] {
            assert_eq!(DecimalText::parse(text).unwrap(), expected, "{text}");
        }
        assert_eq!(DecimalText::from_f64(1.5).unwrap(), expected);
        assert_eq!(DecimalText::parse("0.000").unwrap().digits(), "0");
    }

    #[test]
    fn rounds_once_per_width() {
        let tenth = DecimalText::parse("0.1").unwrap();
        assert_eq!(tenth.to_f64(), Some(0.1));
        assert_eq!(tenth.to_f32(), Some(0.1_f32));
        assert_eq!(DecimalText::parse("1e400").unwrap().to_f64(), None);
        assert_eq!(DecimalText::parse("1e-400").unwrap().to_f64(), Some(0.0));
    }

    #[test]
    fn rejects_malformed_text() {
        for text in ["", ".5", "5.", "1e", "1e+", "1x", "1e9999999999999"] {
            assert!(DecimalText::parse(text).is_none(), "{text}");
        }
    }
}
