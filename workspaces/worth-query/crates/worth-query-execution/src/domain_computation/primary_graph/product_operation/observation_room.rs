//! Room for a product observation the branch refused.
//!
//! A cached Ready's settlement keeps the product observation it reads, and
//! a performed write's retained source keeps the one it was read at. A
//! caller the branch refuses an observation releases those nothing holds,
//! closed cached outputs first and then performed sources nobody holds, one
//! at a time, and asks again after each until the branch admits it. A
//! demand, a read and a mutation commit share the rule, and this is its one
//! site. With none left, the observations belong to open demands and other
//! observers of the World, whose release the registry cannot see: the
//! refusal stands, retryable.

use worth_query_installation::facade::ApplicationSchema;

use crate::basis::WorthQueryProductBranchAdmissionDenial as Denial;
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;

impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    /// Admit a product observation, making room for it from observations
    /// nothing holds. One request's work funds every reclaim of this
    /// admission.
    pub(in crate::domain_computation::primary_graph) fn admit_with_observation_room<Admitted>(
        &self,
        mut admit: impl FnMut() -> Result<Admitted, Denial>,
    ) -> Result<Admitted, Denial> {
        let mut reclaim = None;
        loop {
            match admit() {
                Err(Denial::ObservationCapacityExhausted) => {
                    let admission = reclaim.get_or_insert_with(|| {
                        self.primary_provider
                            .graph
                            .source_owner
                            .invalidation_owner
                            .request_admission()
                    });
                    if !matches!(
                        self.output_demands.reclaim_unheld_observation(admission),
                        Ok(true)
                    ) {
                        return Err(Denial::ObservationCapacityExhausted);
                    }
                }
                answer => return answer,
            }
        }
    }
}
