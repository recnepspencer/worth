//! A provider plan keeps the Home authority of its actual run or bound product.
use super::*;

impl WorthQueryAdmittedProviderExecutionPlan<'_> {
    pub(in crate::domain_computation::provider_session::protocol) fn require_current_home(
        &self,
        current: Option<worth_runtime_world::facade::RuntimeWorldOwnerIdentity>,
    ) -> Result<(), WorthQueryProviderSessionFailure> {
        let presented = match &self.product {
            WorthQueryProviderProductAffinity::Standalone => match &self.run {
                WorthQueryProviderRunCustody::Direct(run) => run.execution_owner_identity(),
                WorthQueryProviderRunCustody::CoupledDirect(owned) => {
                    owned.run().execution_owner_identity()
                }
                WorthQueryProviderRunCustody::Workflow(run) => run.execution_owner_identity(),
            },
            WorthQueryProviderProductAffinity::Application(product) => {
                Some(product.observation().owner_identity())
            }
        };
        if let (Some(current), Some(presented)) = (current, presented) {
            match current.admit_incarnation(presented) {
                Err(worth_proof::HomeIncarnationDenial::StaleIncarnation) => {
                    return Err(failure(
                        WorthQueryProviderSessionDenialKind::StaleIncarnation,
                        "provider plan belongs to an earlier Home incarnation",
                        &self.counters,
                    ));
                }
                Ok(()) | Err(worth_proof::HomeIncarnationDenial::ForeignHome) => {}
            }
        }
        Ok(())
    }
}
