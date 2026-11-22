//! A correction key belongs to the installed caller domain, independently of its target.
use worth_query_declaration::facade::application_operation::ApplicationCanonicalIdentity;

/// Canonically encoded client command key. This descriptive value grants no execution authority.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryCorrectionCommandKey {
    canonical: ApplicationCanonicalIdentity,
}

impl WorthQueryCorrectionCommandKey {
    pub fn from_canonical(
        _: &worth_query_execution_contracts::facade::WorthQueryProgramPublicationAccess,
        canonical: ApplicationCanonicalIdentity,
    ) -> Self {
        Self { canonical }
    }

    pub fn identity(&self) -> [u8; 32] {
        self.canonical.identity()
    }

    pub fn canonical(&self) -> ApplicationCanonicalIdentity {
        self.canonical
    }
}
