//! Descriptive identity embedded once by a complete application build.

/// Identifies one application build without granting execution or recovery authority.
///
/// The build pipeline issues a fresh UUID and embeds it in the application.
/// The resulting artifact keeps that identity across process launches.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ApplicationBuildIdentity([u8; 16]);

impl ApplicationBuildIdentity {
    /// Parses a nonzero UUID in `xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx` form.
    pub fn parse(value: &str) -> Result<Self, InvalidApplicationBuildIdentity> {
        if value.is_empty() {
            return Err(InvalidApplicationBuildIdentity::Missing);
        }
        let source = value.as_bytes();
        if source.len() != 36 {
            return Err(InvalidApplicationBuildIdentity::Malformed);
        }
        let mut bytes = [0; 16];
        let mut digit = 0;
        for (index, byte) in source.iter().copied().enumerate() {
            if matches!(index, 8 | 13 | 18 | 23) {
                if byte != b'-' {
                    return Err(InvalidApplicationBuildIdentity::Malformed);
                }
                continue;
            }
            let nibble = match byte {
                b'0'..=b'9' => byte - b'0',
                b'a'..=b'f' => byte - b'a' + 10,
                b'A'..=b'F' => byte - b'A' + 10,
                _ => return Err(InvalidApplicationBuildIdentity::Malformed),
            };
            bytes[digit / 2] = (bytes[digit / 2] << 4) | nibble;
            digit += 1;
        }
        if bytes == [0; 16] {
            return Err(InvalidApplicationBuildIdentity::Zero);
        }
        Ok(Self(bytes))
    }

    /// Returns the portable descriptive bytes of the embedded identity.
    pub const fn bytes(self) -> [u8; 16] {
        self.0
    }
}

/// An absent or invalid build identity cannot enter ordinary installation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InvalidApplicationBuildIdentity {
    Missing,
    Malformed,
    Zero,
}

impl std::fmt::Display for InvalidApplicationBuildIdentity {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::Missing => "application build identity is missing",
            Self::Malformed => "application build identity must be a UUID",
            Self::Zero => "application build identity must be nonzero",
        })
    }
}

impl std::error::Error for InvalidApplicationBuildIdentity {}

#[cfg(test)]
mod tests {
    use super::{ApplicationBuildIdentity as Build, InvalidApplicationBuildIdentity as Invalid};

    #[test]
    fn pipeline_uuid_is_exact_and_nonzero() {
        assert_eq!(Build::parse(""), Err(Invalid::Missing));
        for value in [
            "1",
            "11111111111111111111111111111111",
            " 11111111-1111-1111-1111-111111111111",
            "gggggggg-1111-1111-1111-111111111111",
        ] {
            assert_eq!(Build::parse(value), Err(Invalid::Malformed));
        }
        assert_eq!(
            Build::parse("00000000-0000-0000-0000-000000000000"),
            Err(Invalid::Zero)
        );
        assert_eq!(
            Build::parse("11111111-1111-1111-1111-111111111111")
                .unwrap()
                .bytes(),
            [0x11; 16]
        );
        assert_eq!(
            Build::parse("ABCDEF12-3456-7890-abcd-ef1234567890"),
            Build::parse("abcdef12-3456-7890-abcd-ef1234567890")
        );
    }
}
