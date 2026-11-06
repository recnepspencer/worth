//! The same SDK tree retains the actual kind of Native parent custody.
use super::*;
use worth_query_execution::publication_boundary::document_history::{
    WorthQuerySettledHistoryProgramOutput, WorthQueryVerifiedHistoryParent,
};

/// Sealed traversal custody. Neither kind can be converted into the other.
#[doc(hidden)]
pub struct ProgramOutputParentAuthority<S, P, D>
where
    S: ApplicationSchema,
    P: ApplicationProgramDefinition<S>,
    D: WorthQueryApplicationOutputDemand<S>,
{
    kind: ParentAuthorityKind<S, P, D>,
}

pub(in crate::application_entry) enum ParentAuthorityKind<S, P, D>
where
    S: ApplicationSchema,
    P: ApplicationProgramDefinition<S>,
    D: WorthQueryApplicationOutputDemand<S>,
{
    Ordinary(std::sync::Arc<WorthQuerySettledProgramOutput<S, P, D>>),
    History(std::sync::Arc<WorthQuerySettledHistoryProgramOutput<S, P, D>>),
    Verified(WorthQueryVerifiedHistoryParent<S, P, D>),
}

impl<S, P, D> ProgramOutputParentAuthority<S, P, D>
where
    S: ApplicationSchema,
    P: ApplicationProgramDefinition<S>,
    D: WorthQueryApplicationOutputDemand<S>,
{
    pub(in crate::application_entry) fn ordinary(
        value: WorthQuerySettledProgramOutput<S, P, D>,
    ) -> Self {
        Self {
            kind: ParentAuthorityKind::Ordinary(std::sync::Arc::new(value)),
        }
    }
    pub(in crate::application_entry) fn history(
        value: WorthQuerySettledHistoryProgramOutput<S, P, D>,
    ) -> Self {
        Self {
            kind: ParentAuthorityKind::History(std::sync::Arc::new(value)),
        }
    }
    pub(in crate::application_entry) fn verified(
        value: WorthQueryVerifiedHistoryParent<S, P, D>,
    ) -> Self {
        Self {
            kind: ParentAuthorityKind::Verified(value),
        }
    }
    pub(in crate::application_entry) fn kind(&self) -> &ParentAuthorityKind<S, P, D> {
        &self.kind
    }
}

impl<S, P, D> Clone for ProgramOutputParentAuthority<S, P, D>
where
    S: ApplicationSchema,
    P: ApplicationProgramDefinition<S>,
    D: WorthQueryApplicationOutputDemand<S>,
{
    fn clone(&self) -> Self {
        Self {
            kind: match &self.kind {
                ParentAuthorityKind::Ordinary(value) => {
                    ParentAuthorityKind::Ordinary(std::sync::Arc::clone(value))
                }
                ParentAuthorityKind::Verified(value) => {
                    ParentAuthorityKind::Verified(value.clone())
                }
                ParentAuthorityKind::History(value) => {
                    ParentAuthorityKind::History(std::sync::Arc::clone(value))
                }
            },
        }
    }
}
