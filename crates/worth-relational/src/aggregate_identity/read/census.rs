//! Complete exact-view census; individual reads never depend on enumeration.
use super::*;
use crate::aggregate_identity::{
    funding, retention_extent as extent, selection::AggregateSelection,
};
use crate::identity::data::KindId;
use crate::validation::engine::state_view::InvariantStateView;
use std::collections::BTreeMap;
use worth_execution::ChargedBytes;

/// Whole installed-definition census in canonical Native aggregate identity order.
/// Every body retains its actual Native payer; collection backing remains on the
/// original invariant payer. Census deliberately examines the exact-view entity
/// inventory, including unrelated kinds/retired slots, and can grow with it.
pub struct AggregateReadCollection {
    groups: BTreeMap<worth_foundational::facade::AggregateIdentity, AggregateRead>,
    _retention: crate::validation::CustomInvariantWorkMeter,
}
impl AggregateReadCollection {
    pub fn len(&self) -> usize {
        self.groups.len()
    }
    pub fn is_empty(&self) -> bool {
        self.groups.is_empty()
    }
    pub fn iter(&self) -> impl ExactSizeIterator<Item = &AggregateRead> {
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
        for &kind in contract.source.relations() {
            budget.check_live().map_err(AggregateReadDenial::Budget)?;
            self.view
                .require_relation_kind(kind)
                .map_err(|cause| AggregateReadDenial::Budget(budget.refusal(cause)))?;
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
        let mut covered = BTreeSet::new();
        let mut result = AggregateReadCollection {
            groups: BTreeMap::new(),
            _retention: self.view.work_meter(),
        };
        for &member in &universe.0 {
            budget
                .checkpoint(search_work(covered.len()))
                .map_err(AggregateReadDenial::Budget)?;
            if covered.contains(&member) {
                continue;
            }
            let group = self.of_definition(definition, member)?;
            for actual in group.members() {
                budget
                    .checkpoint(
                        search_work(universe.0.len())
                            .saturating_add(search_work(covered.len()).saturating_mul(2)),
                    )
                    .map_err(AggregateReadDenial::Budget)?;
                if universe.0.binary_search(&actual).is_err() || covered.contains(&actual) {
                    return Err(inconsistent(
                        source,
                        "census cohort differs from exact member universe",
                    ));
                }
                budget
                    .claim_retained(extent::seed())
                    .map_err(AggregateReadDenial::Budget)?;
                covered.insert(actual);
            }
            budget
                .checkpoint(search_work(result.groups.len()).saturating_mul(2))
                .map_err(AggregateReadDenial::Budget)?;
            if result.groups.contains_key(&group.identity()) {
                return Err(inconsistent(
                    source,
                    "distinct census cohorts share an identity",
                ));
            }
            budget
                .claim_retained(extent::tree_entry::<
                    worth_foundational::facade::AggregateIdentity,
                    AggregateRead,
                >())
                .map_err(AggregateReadDenial::Budget)?;
            result.groups.insert(group.identity(), group);
        }
        if covered.len() != universe.0.len() {
            return Err(inconsistent(source, "census coverage is incomplete"));
        }
        budget.check_live().map_err(AggregateReadDenial::Budget)?;
        Ok(result)
    }
}
struct VisibleMembers(Vec<EntityId>);
impl ChargedBytes for VisibleMembers {
    fn additional_charged_bytes(&self) -> u64 {
        self.0.capacity() as u64 * std::mem::size_of::<EntityId>() as u64
    }
}
fn visible_members(
    state: &InvariantStateView<'_>,
    kind: KindId,
    work: &mut funding::AggregateWork<'_, '_, '_, '_>,
) -> funding::BuildResult<VisibleMembers> {
    work.with_partition_ids(state.state(), |partitions, work| {
        let mut members = Vec::new();
        for &partition in partitions {
            work.checkpoint(1)?;
            work.visit_entity_slots(state, partition, |slot, work| {
                work.checkpoint(1)?;
                if let Some(metadata) = state.entity_metadata_for_slot(partition, slot) {
                    if metadata.kind_id == kind {
                        work.retain(extent::vector_entry::<EntityId>())?;
                        if members
                            .last()
                            .is_some_and(|previous| *previous >= metadata.entity_id)
                        {
                            return work.invalid(funding::PlanDenial::InvalidComponentGraph);
                        }
                        members.push(metadata.entity_id);
                    }
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
