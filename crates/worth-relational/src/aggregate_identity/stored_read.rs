//! Strict stored groups use the same funded Native permutation traversal and decoder.
use super::funding::PlanDenial;
use super::{cohorts, funding, retention_extent as extent};
use crate::identity::data::EntityId;
use crate::schema::data::AggregateIdentityContractDeclaration;
use crate::transactions::data::TransactionCommitError;
use crate::validation::engine::state_view::InvariantStateView;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;
use worth_execution::{ChargedBytes, ExecutionMemoryReservation};
use worth_foundational::facade::AggregateIdentity;

pub(super) struct StoredAggregateRead {
    pub(super) contract: AggregateIdentityContractDeclaration,
    pub(super) identity: AggregateIdentity,
    pub(super) members: BTreeSet<EntityId>,
    _retention: Option<Arc<ExecutionMemoryReservation>>,
}
struct StoredValue {
    identity: AggregateIdentity,
    members: BTreeSet<EntityId>,
}
impl ChargedBytes for StoredValue {
    fn additional_charged_bytes(&self) -> u64 {
        self.members.len() as u64 * extent::tree_entry::<EntityId, ()>()
            + extent::arc_backing::<StoredAggregateRead>()
            + extent::arc_backing::<ExecutionMemoryReservation>()
    }
}

pub(super) fn in_custom_scope(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    view: &InvariantStateView<'_>,
    contract: AggregateIdentityContractDeclaration,
    member: EntityId,
    allowance: &mut dyn FnMut(u64) -> Result<(), crate::execution::RelationalExecutionDenialCause>,
) -> Result<Arc<StoredAggregateRead>, TransactionCommitError> {
    let (value, retention) = funding::run_in_custom_read(execution, allowance, |work| {
        work.retain(
            extent::arc_backing::<StoredAggregateRead>()
                + extent::arc_backing::<ExecutionMemoryReservation>(),
        )?;
        work.retain(extent::seed())?;
        let seeds = BTreeSet::from([member]);
        let mut groups = cohorts::groups(&seeds, view, contract, &BTreeMap::new(), work)?;
        if groups.len() != 1 {
            return work.invalid(PlanDenial::InvalidPermutation);
        }
        let members = groups.pop().expect("one complete group checked");
        let stored =
            cohorts::read_stored_group(view, &members, contract, &mut BTreeMap::new(), work)?;
        Ok(StoredValue {
            identity: stored.record.identity,
            members,
        })
    })?;
    Ok(Arc::new(StoredAggregateRead {
        contract,
        identity: value.identity,
        members: value.members,
        _retention: retention.map(Arc::new),
    }))
}
