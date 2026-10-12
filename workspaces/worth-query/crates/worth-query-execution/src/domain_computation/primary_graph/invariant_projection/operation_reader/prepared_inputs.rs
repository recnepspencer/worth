//! A prepared operation's realized upstreams, sealed by native current reads.
use super::WorthQueryApplicationOperationInvariantProjectionSnapshot;
use crate::domain_computation::primary_graph::invariant_projection::ConsumedOutputEvidence;
use worth_query_installation::facade::ApplicationSchema;

/// Only an operation projection can lend these already verified inputs. The
/// required-set owner checks their committed rows before admitting compute.
pub(in crate::domain_computation::primary_graph) struct PreparedCurrentInputs<'a> {
    product: &'a crate::basis::WorthQueryProductBranchLease,
    consumed: &'a std::collections::BTreeMap<
        std::sync::Arc<
            crate::domain_computation::primary_graph::output_lineage::RecordedSettlementIdentity,
        >,
        ConsumedOutputEvidence,
    >,
}
impl PreparedCurrentInputs<'_> {
    pub(in crate::domain_computation::primary_graph) fn native_basis(&self) -> (u64, u64) {
        let basis = self.product.relational_basis_descriptor();
        (basis.runtime_instance_id(), basis.root_identity())
    }
    pub(in crate::domain_computation::primary_graph) fn consumed(
        &self,
    ) -> impl ExactSizeIterator<Item = &ConsumedOutputEvidence> {
        self.consumed.values()
    }
}
impl<Schema: ApplicationSchema, Operation>
    WorthQueryApplicationOperationInvariantProjectionSnapshot<Schema, Operation>
{
    pub(in crate::domain_computation::primary_graph) fn prepared_current_inputs(
        &self,
    ) -> PreparedCurrentInputs<'_> {
        PreparedCurrentInputs {
            product: &self.product,
            consumed: &self.snapshot.consumed_outputs,
        }
    }
}
