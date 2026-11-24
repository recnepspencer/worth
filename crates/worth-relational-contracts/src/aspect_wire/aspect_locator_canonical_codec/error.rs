#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AspectValueLocatorCanonicalCodecError {
    detail: String,
}

impl AspectValueLocatorCanonicalCodecError {
    pub(super) fn new(detail: impl Into<String>) -> Self {
        Self {
            detail: detail.into(),
        }
    }
}

impl std::fmt::Display for AspectValueLocatorCanonicalCodecError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.detail.fmt(formatter)
    }
}

impl std::error::Error for AspectValueLocatorCanonicalCodecError {}
