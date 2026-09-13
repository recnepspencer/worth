use core::error::Error;
use core::fmt;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InvalidName {
    reason: &'static str,
}

impl InvalidName {
    fn new(reason: &'static str) -> Self {
        Self { reason }
    }
}

impl fmt::Display for InvalidName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.reason)
    }
}

impl Error for InvalidName {}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Name(String);

impl Name {
    pub fn new(raw: impl Into<String>) -> Result<Self, InvalidName> {
        let raw = raw.into();
        validate_name(&raw)?;
        Ok(Self(raw))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

fn validate_name(raw: &str) -> Result<(), InvalidName> {
    if raw.is_empty() {
        return Err(InvalidName::new("name must not be empty"));
    }
    if !raw
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
    {
        return Err(InvalidName::new(
            "name must use only ascii letters, digits, '-' or '_'",
        ));
    }
    Ok(())
}
