/// Construction failed before a Query product runtime became callable.
#[derive(Debug)]
pub struct WorthQueryProductRuntimeInstallationDenial {
    detail: String,
    kind: WorthQueryProductRuntimeInstallationDenialKind,
}

impl WorthQueryProductRuntimeInstallationDenial {
    pub(super) fn new(detail: String) -> Self {
        Self {
            detail,
            kind: WorthQueryProductRuntimeInstallationDenialKind::Installation,
        }
    }

    pub const fn kind(&self) -> WorthQueryProductRuntimeInstallationDenialKind {
        self.kind
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryProductRuntimeInstallationDenialKind {
    Installation,
    Handle(crate::facade::primary_graph::WorthQueryHandleDenial),
}
impl From<crate::facade::primary_graph::WorthQueryHandleDenial>
    for WorthQueryProductRuntimeInstallationDenial
{
    fn from(denial: crate::facade::primary_graph::WorthQueryHandleDenial) -> Self {
        Self {
            detail: denial.to_string(),
            kind: WorthQueryProductRuntimeInstallationDenialKind::Handle(denial),
        }
    }
}
