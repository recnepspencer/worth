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

use crate::basis::WorthQueryProductBranchAdmissionRefusalKind as AdmissionRefusal;
use worth_query_installation::facade::ApplicationSchema;

use crate::basis::WorthQueryProductBranchAdmissionDenial as Denial;

pub(in crate::domain_computation::primary_graph) trait ObservationRoomScopedOps<
    Schema: ApplicationSchema,
>:
    crate::domain_computation::primary_graph::product_operation::host::ProductOperationHost<Schema>
{
    fn admit_with_observation_room<Admitted>(
        &self,
        mut admit: impl FnMut() -> Result<Admitted, Denial>,
    ) -> Result<Admitted, Denial> {
        loop {
            match admit() {
                Err(Denial::Refused(AdmissionRefusal::ObservationCapacityExhausted)) => {
                    if !matches!(self.output_demands().reclaim_unheld_observation(), Ok(true)) {
                        return Err(Denial::Refused(
                            AdmissionRefusal::ObservationCapacityExhausted,
                        ));
                    }
                }
                answer => return answer,
            }
        }
    }
}

impl<
        Schema: ApplicationSchema,
        HostRuntime: crate::domain_computation::primary_graph::product_operation::host::ProductOperationHost<
                Schema,
            > + ?Sized,
    > ObservationRoomScopedOps<Schema> for HostRuntime
{
}
