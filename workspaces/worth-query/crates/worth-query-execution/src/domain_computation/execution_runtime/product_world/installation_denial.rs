/// Construction failed before a Query product runtime became callable.
#[derive(Debug)]
pub struct WorthQueryProductRuntimeInstallationDenial {
    detail: String,
}

impl WorthQueryProductRuntimeInstallationDenial {
    pub(super) fn new(detail: String) -> Self {
        Self { detail }
    }

    pub fn detail(&self) -> &str {
        &self.detail
    }
}

impl std::fmt::Display for WorthQueryProductRuntimeInstallationDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.detail)
    }
}

impl std::error::Error for WorthQueryProductRuntimeInstallationDenial {}
