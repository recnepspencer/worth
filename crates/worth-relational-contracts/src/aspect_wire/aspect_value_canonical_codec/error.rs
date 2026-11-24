#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AspectValueCanonicalCodecError {
    detail: String,
}

impl AspectValueCanonicalCodecError {
    pub fn new(detail: impl Into<String>) -> Self {
        Self {
            detail: detail.into(),
        }
    }

    pub fn detail(&self) -> &str {
        &self.detail
    }
}

impl std::fmt::Display for AspectValueCanonicalCodecError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.detail)
    }
}

impl std::error::Error for AspectValueCanonicalCodecError {}
