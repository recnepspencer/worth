use serde::{Deserialize, Serialize};

/// Portable interpretation coordinate, never byte availability or read authority.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ContentFormat {
    name: String,
    version: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContentFormatDenial {
    InvalidName,
    ZeroVersion,
}

impl ContentFormat {
    pub fn new(name: impl Into<String>, version: u32) -> Result<Self, ContentFormatDenial> {
        let format = Self {
            name: name.into(),
            version,
        };
        format.validate()?;
        Ok(format)
    }

    /// Check also at trust crossings: a decoded descriptor remains descriptive.
    pub fn validate(&self) -> Result<(), ContentFormatDenial> {
        Self::validate_parts(&self.name, self.version)
    }

    pub fn validate_parts(name: &str, version: u32) -> Result<(), ContentFormatDenial> {
        let bytes = name.as_bytes();
        if !(1..=256).contains(&bytes.len())
            || !bytes[0].is_ascii_lowercase()
            || !bytes.iter().all(|byte| {
                byte.is_ascii_lowercase()
                    || byte.is_ascii_digit()
                    || matches!(byte, b'.' | b'_' | b'/' | b'-')
            })
        {
            return Err(ContentFormatDenial::InvalidName);
        }
        if version == 0 {
            return Err(ContentFormatDenial::ZeroVersion);
        }
        Ok(())
    }

    pub fn name(&self) -> &str {
        &self.name
    }
    pub const fn version(&self) -> u32 {
        self.version
    }
}
