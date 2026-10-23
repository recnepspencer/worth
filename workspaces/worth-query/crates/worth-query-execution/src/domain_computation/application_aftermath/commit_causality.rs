//! Recorded correction evidence joins the ordinary publication's fresh admission.
use super::WorthQueryPendingAftermathCausality;
use crate::domain_computation::primary_graph::{
    WorthQueryAdmittedApplicationOperation, WorthQueryApplicationCommitDenial,
};
use worth_query_installation::facade::ApplicationSchema;

pub(crate) enum ApplicationCommitCausality {
    Ordinary,
    RecordedCorrection(
        std::sync::Arc<
            crate::domain_computation::primary_graph::WorthQueryPreparedCorrectionSource,
        >,
    ),
}

impl ApplicationCommitCausality {
    pub(crate) fn admit<Schema: ApplicationSchema, Operation, Input, Scope>(
        self,
        admission: &WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
    ) -> Result<Option<WorthQueryPendingAftermathCausality>, WorthQueryApplicationCommitDenial>
    {
        match self {
            Self::Ordinary => Ok(None),
            Self::RecordedCorrection(source) => {
                if !source.matches_admission(admission) {
                    return Err(WorthQueryApplicationCommitDenial::correction_context_mismatch());
                }
                Ok(Some(
                    WorthQueryPendingAftermathCausality::recorded_correction(source),
                ))
            }
        }
    }
}
