//! Sealed recovery custody joins the ordinary publication's current admission.
use super::{
    recovery_progression::WorthQueryRecoveryBindingCurrentTruth,
    WorthQueryPendingAftermathCausality, WorthQueryRecoveryHandle,
    WorthQueryRecoveryHandleDenialKind, WorthQueryRedoProgressionHandoff,
    WorthQueryUndoProgressionHandoff,
};
use crate::domain_computation::primary_graph::{
    WorthQueryAdmittedApplicationOperation, WorthQueryApplicationCommitDenial,
};
use worth_query_installation::facade::ApplicationSchema;

pub(crate) enum ApplicationCommitCausality<'handoff> {
    Ordinary,
    Aftermath {
        pending: WorthQueryPendingAftermathCausality,
        handle: &'handoff WorthQueryRecoveryHandle,
    },
}

impl<'handoff> ApplicationCommitCausality<'handoff> {
    pub(crate) fn undo(handoff: &'handoff WorthQueryUndoProgressionHandoff) -> Self {
        Self::Aftermath {
            pending: handoff.pending_causality(),
            handle: handoff.admission().recovery_handle(),
        }
    }
    pub(crate) fn redo(handoff: &'handoff WorthQueryRedoProgressionHandoff) -> Self {
        Self::Aftermath {
            pending: handoff.pending_causality(),
            handle: handoff.recovery_handle(),
        }
    }
    pub(crate) fn admit<Schema: ApplicationSchema, Operation, Input, Scope>(
        self,
        admission: &WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
    ) -> Result<Option<WorthQueryPendingAftermathCausality>, WorthQueryApplicationCommitDenial>
    {
        match self {
            Self::Ordinary => Ok(None),
            Self::Aftermath { pending, handle } => {
                if admission.runtime_authority() != handle.runtime_authority() {
                    return Err(
                        WorthQueryApplicationCommitDenial::recovery_handoff_mismatch(
                            WorthQueryRecoveryHandleDenialKind::FreshAuthorityDenied,
                        ),
                    );
                }
                WorthQueryRecoveryBindingCurrentTruth::from_admitted(admission)
                    .and_then(|current| current.check(handle.binding()))
                    .map_err(|stop| {
                        WorthQueryApplicationCommitDenial::recovery_handoff_mismatch(stop.kind())
                    })?;
                Ok(Some(pending))
            }
        }
    }
}
