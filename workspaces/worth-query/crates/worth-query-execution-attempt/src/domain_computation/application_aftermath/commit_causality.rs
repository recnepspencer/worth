//! Recorded correction evidence joins the ordinary publication's fresh admission.
use super::WorthQueryPendingAftermathCausality;
use crate::domain_computation::primary_graph::{
    WorthQueryAdmittedApplicationOperation, WorthQueryApplicationCommitDenial,
};
use worth_query_installation::facade::ApplicationSchema;

pub enum ApplicationCommitCausality {
    Ordinary,
    RecordedCorrection(
        std::sync::Arc<
            dyn crate::ports::PreparedCorrectionSourcePort,
        >,
    ),
}

impl ApplicationCommitCausality {
    pub fn admit<Schema: ApplicationSchema, Operation, Input, Scope>(
        self,
        admission: &WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
    ) -> Result<Option<WorthQueryPendingAftermathCausality>, WorthQueryApplicationCommitDenial>
    {
        match self {
            Self::Ordinary => Ok(None),
            Self::RecordedCorrection(source) => {
                let product = admission
                    .graph_work()
                    .mutation_product()
                    .ok_or_else(WorthQueryApplicationCommitDenial::correction_context_mismatch)?;
                source.admit_receiver(product.observation().owner_identity())?;
                let target = admission.graph_work().mutation_product().map(|p| crate::basis::WorthQueryProductBranchReadIdentity::from_observation(p.observation()));
                if !source.matches_admission_basis(
                    admission.runtime_authority(),
                    &admission.operation_scope_binding().principal(),
                    &admission.operation_scope_binding().scope(),
                    target.as_ref(),
                ) {
                    return Err(WorthQueryApplicationCommitDenial::correction_context_mismatch());
                }
                Ok(Some(
                    WorthQueryPendingAftermathCausality::recorded_correction(source),
                ))
            }
        }
    }
}
