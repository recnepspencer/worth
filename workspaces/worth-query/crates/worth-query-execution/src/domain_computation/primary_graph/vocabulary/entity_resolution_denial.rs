//! `entity_resolution_denial`'s vocabulary, moved whole from `primary_graph::entity_resolution_denial`: its root file's items here, one child per other source file.
use crate::domain_computation::vocabulary::primary_graph::entity_resolution_denial::WorthQueryEntityResolutionDenialKind;



/// Refusal to resolve an entity from its identity field.
///
/// Nothing was read beyond the lookup. [`Self::kind`] says why and
/// [`Self::subject`] names the entity or field.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryEntityResolutionDenial {
    kind: WorthQueryEntityResolutionDenialKind,
    subject: String,
}

impl WorthQueryEntityResolutionDenial {
    pub(in crate::domain_computation::primary_graph) fn new(
        kind: WorthQueryEntityResolutionDenialKind,
        subject: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            subject: subject.into(),
        }
    }

    pub const fn kind(&self) -> WorthQueryEntityResolutionDenialKind {
        self.kind
    }

    pub fn subject(&self) -> &str {
        &self.subject
    }
}

impl std::fmt::Display for WorthQueryEntityResolutionDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "application entity resolution denied: {:?} ({})",
            self.kind, self.subject
        )
    }
}

impl std::error::Error for WorthQueryEntityResolutionDenial {}

impl worth_execution::ChargedBytes for WorthQueryEntityResolutionDenial {
    fn additional_charged_bytes(&self) -> u64 {
        self.subject.capacity() as u64
    }
}
