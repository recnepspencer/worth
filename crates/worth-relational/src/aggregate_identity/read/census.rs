//! Complete exact-view census; individual reads never depend on enumeration.
use super::*;
use crate::aggregate_identity::{
    funding, retention_extent as extent, selection::AggregateSelection,
};
use crate::identity::data::KindId;
use crate::validation::engine::state_view::InvariantStateView;
use std::collections::BTreeMap;
use worth_execution::ChargedBytes;
mod collection;
mod projected;
mod proof;
pub use proof::{AggregateCensusRead, AggregateCensusReadIdentity, PreparedAggregateCensus};

/// Whole installed-definition census in canonical Native aggregate identity order.
/// Every body retains its actual Native payer; collection backing remains on the
/// original invariant payer. Census deliberately examines the exact-view entity
/// inventory, including unrelated kinds/retired slots, and can grow with it.
pub struct AggregateReadCollection<Projection = EntityId> {
    groups: BTreeMap<worth_foundational::facade::AggregateIdentity, AggregateRead<Projection>>,
    _retention: crate::validation::CustomInvariantWorkMeter,
}
impl<Projection> AggregateReadCollection<Projection> {
    pub fn len(&self) -> usize {
        self.groups.len()
    }
    pub fn is_empty(&self) -> bool {
        self.groups.is_empty()
    }
    pub fn iter(&self) -> impl ExactSizeIterator<Item = &AggregateRead<Projection>> {
        self.groups.values()
    }
}
impl AggregateReadPort<'_, '_> {
    /// Read all genuine cohorts, or refuse without a partial successful census.
    /// Empty sets still validate member and every source-kind access first.
    pub fn all_of_definition(
        &self,
        definition: AggregateDefinitionIdentity,
    ) -> Result<AggregateReadCollection, AggregateReadDenial> {
        let budget = self.view.read_budget();
        budget.check_live().map_err(AggregateReadDenial::Budget)?;
        let (state, execution, source) = self.view.aggregate_read_parts();
        let (source, committed) = source.ok_or(AggregateReadDenial::SourceUnavailable)?;
        budget
            .checkpoint(source.contract_count(committed).saturating_add(1))
            .map_err(AggregateReadDenial::Budget)?;
        let contract = source
            .contract(AggregateSelection::Definition(definition), committed)
            .ok_or(AggregateReadDenial::NotAggregateRelation)?;
        self.view
            .require_entity_kind(contract.member_kind)
            .map_err(|cause| AggregateReadDenial::Budget(budget.refusal(cause)))?;
        let derived = self.require_source(contract, source.registry(committed))?;
        if derived
            .as_ref()
            .is_some_and(|source| source.parts().from.entity_kind().is_none())
        {
            return Err(AggregateReadDenial::WrongMemberKind);
        }
        let mut allowance = |units| budget.debit_native_rule_work(units);
        let (universe, _universe_retention) =
            funding::run_custom_read(&execution, &mut allowance, |work| {
                visible_members(state, contract.member_kind, work)
            })
            .map_err(|error| {
                source.latch_failure(error.clone());
                from_preparation(error)
            })?;
        collection::read(self, source, definition, universe)
    }
}

struct VisibleMembers(Vec<NativeMemberKey>);
impl ChargedBytes for VisibleMembers {
    fn additional_charged_bytes(&self) -> u64 {
        self.0.capacity() as u64 * std::mem::size_of::<NativeMemberKey>() as u64
    }
}
fn visible_members(
    state: &InvariantStateView<'_>,
    kind: KindId,
    work: &mut funding::AggregateWork<'_, '_, '_, '_>,
) -> funding::BuildResult<VisibleMembers> {
    work.with_partition_ids(state.state(), |partitions, work| {
        let mut members: Vec<NativeMemberKey> = Vec::new();
        for &partition in partitions {
            work.checkpoint(1)?;
            work.visit_entity_slots(state, partition, |slot, work| {
                work.checkpoint(1)?;
                if let Some(metadata) = state.entity_metadata_for_slot(partition, slot)
                    && metadata.kind_id == kind
                {
                    work.retain(extent::vector_entry::<NativeMemberKey>())?;
                    if members
                        .last()
                        .is_some_and(|previous| previous.carrier() >= metadata.entity_id)
                    {
                        return work.invalid(funding::PlanDenial::InvalidComponentGraph);
                    }
                    members.push(NativeMemberKey::Entity(metadata.entity_id));
                }
                Ok(())
            })?;
        }
        // Selected partition IDs and slots are canonical; generation is supplied
        // by that exact view's existing lifecycle/base/staged visibility owner.
        work.poll()?;
        Ok(VisibleMembers(members))
    })
}

// Match the existing Native aggregate tree-lookup debit convention. Work is
// observed on the original rule/request owner, never a new census allowance.
fn search_work(entries: usize) -> u64 {
    (usize::BITS - entries.max(1).leading_zeros()) as u64 * 22 + 1
}
