use core::fmt;

/// Stable machine-readable identity assigned by the boundary that owns a denial.
///
/// Codes begin with a lowercase ASCII letter and contain only lowercase ASCII
/// letters, digits, dots, underscores, and hyphens. Use fixed, namespaced codes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ReportCode(&'static str);

impl ReportCode {
    /// Validates a static code. Use this in a constant to check it at compile time.
    pub const fn new(code: &'static str) -> Result<Self, InvalidReportCode> {
        let [first, remaining @ ..] = code.as_bytes() else {
            return Err(InvalidReportCode);
        };
        if !first.is_ascii_lowercase() {
            return Err(InvalidReportCode);
        }
        let mut index = 0;
        while index < remaining.len() {
            let byte = remaining[index];
            if !matches!(byte, b'a'..=b'z' | b'0'..=b'9' | b'.' | b'_' | b'-') {
                return Err(InvalidReportCode);
            }
            index += 1;
        }
        Ok(Self(code))
    }

    pub const fn as_str(self) -> &'static str {
        self.0
    }
}

impl fmt::Display for ReportCode {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.0)
    }
}

/// A static code failed the [`ReportCode`] grammar.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InvalidReportCode;

impl fmt::Display for InvalidReportCode {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("invalid report code")
    }
}

impl core::error::Error for InvalidReportCode {}
