//! The successor guards (#119): the conditions under which retracting a
//! chain's live round and applying its next round as written equals the
//! one-round answer. Every check reads only live members' entries.
use super::indexes::{MemberId, MemberIndexes};
use super::ledger::ChainCreateLedger;
use contracts::identity::data::record_ids::EntityId;
use contracts::lineage::data::CommitNamingPlan;
use contracts::mvcc::publication::application_publication::output_meaning::RequiredOutputRoleTarget;
use contracts::transactions::data::intents::mutation_intent::{
    CreateIntent, EntityMutationIntent, MutationIntent, RelationMutationIntent,
};
use contracts::transactions::data::primitives::{EntityReference, RecordRef};
use worth_relational_contracts::internal_port as contracts;

/// The refusal subject for `record`: the entity itself, or the route source
/// for a relation.
pub fn subject(record: &RecordRef, source: EntityId) -> EntityId {
    match record {
        RecordRef::Entity(entity) => *entity,
        RecordRef::Relation(_) => source,
    }
}

/// The first of `records` a live member other than `member` wrote after the
/// chain's head (`IntervenedOutput`). Backward: over the chain's touched set;
/// forward (Option A): over the new round's changed records.
/// O(|records| · log n).
pub fn intervened<'a>(
    indexes: &MemberIndexes,
    records: impl IntoIterator<Item = &'a RecordRef>,
    member: MemberId,
    head_ordinal: u64,
) -> Option<&'a RecordRef> {
    records
        .into_iter()
        .find(|record| indexes.written_after(record, member, head_ordinal))
}

/// Every existing record a member names: its intents' targets and existing
/// endpoints, its existing role targets, and its naming plan's records.
/// O(intents + roles + plan).
pub fn named_records<'a>(
    intents: impl IntoIterator<Item = &'a MutationIntent>,
    roles: &[RequiredOutputRoleTarget],
    naming: Option<&CommitNamingPlan>,
) -> Vec<RecordRef> {
    let mut named = Vec::new();
    for intent in intents {
        visit_record_ids(intent, |record| named.push(record));
    }
    for role in roles {
        if let EntityReference::Existing(id) = &role.target {
            named.push(RecordRef::Entity(*id));
        }
    }
    if let Some(plan) = naming {
        super::super::part_naming::visit_all_record_refs(plan, |record| named.push(record));
    }
    named
}

/// A record the new round names that only a superseded round of its own
/// chain created (`PriorEffectRead`): it can only have been read from that
/// superseded effect.
pub fn stale<'a>(ledger: &ChainCreateLedger, named: &'a [RecordRef]) -> Option<&'a RecordRef> {
    named.iter().find(|record| ledger.is_dormant(record))
}

/// A record no live round of the chain creates any more that another live
/// member still names (`RetiredReference`).
pub fn retired<'a>(
    ledger: &'a ChainCreateLedger,
    indexes: &MemberIndexes,
    member: MemberId,
) -> Option<&'a RecordRef> {
    ledger
        .dormant()
        .find(|record| indexes.referenced_by_other(record, member))
}

/// Option A's forward guard, then the stale-base guard, over the records
/// part `index`'s applied round changed (§2.3 step 8; Lean `StaleBaseOk`).
pub fn applied_round<'a>(
    parts: &[super::super::retained_assembly::CoupledPublicationPart],
    indexes: &MemberIndexes,
    index: usize,
    records: &'a [RecordRef],
) -> Option<&'a RecordRef> {
    let head = parts[index].chain.head_ordinal;
    intervened(indexes, records, MemberId::Part(index), head).or_else(|| {
        let others = parts
            .iter()
            .enumerate()
            .filter(|(other, _)| *other != index);
        let others = others.filter_map(|(other, part)| {
            Some(OtherChain {
                head_ordinal: part.chain.head_ordinal,
                live_ordinal: indexes.ordinal(MemberId::Part(other))?,
                touched: &part.chain.touched,
            })
        });
        stale_base(records, head, others)
    })
}

/// One other chain as the stale-base guard reads it.
pub struct OtherChain<'a> {
    pub head_ordinal: u64,
    pub live_ordinal: u64,
    pub touched: &'a std::collections::BTreeSet<RecordRef>,
}

/// The first of the round's `records` that an earlier-headed chain touched
/// and re-rounded after this chain's head (`IntervenedOutput`). Such a chain
/// may have retracted the record past the image this chain's head saw, so a
/// later retraction of this chain would restore a superseded image. Without
/// this guard the live result can differ from the one-round answer (Lean
/// `MultiChainGap.two_guards_insufficient`); with it, `T4m` holds.
/// O(|records| · chains · log n).
pub fn stale_base<'a, 'c>(
    records: &'a [RecordRef],
    head_ordinal: u64,
    others: impl IntoIterator<Item = OtherChain<'c>> + Clone,
) -> Option<&'a RecordRef> {
    records.iter().find(|record| {
        others.clone().into_iter().any(|other| {
            other.head_ordinal < head_ordinal
                && other.live_ordinal > head_ordinal
                && other.touched.contains(record)
        })
    })
}

/// Every existing record `intent` names: its write target and each
/// `Existing` relation endpoint. Created references name no record yet.
/// O(endpoints).
fn visit_record_ids(intent: &MutationIntent, mut visit: impl FnMut(RecordRef)) {
    let visit = &mut visit;
    let endpoint = |reference: &EntityReference, visit: &mut dyn FnMut(RecordRef)| {
        if let EntityReference::Existing(id) = reference {
            visit(RecordRef::Entity(*id));
        }
    };
    match intent {
        MutationIntent::Create(CreateIntent::Relation(spec)) => {
            endpoint(&spec.source, visit);
            endpoint(&spec.target, visit);
        }
        MutationIntent::Create(CreateIntent::RelationAspects(spec)) => {
            endpoint(&spec.source, visit);
            endpoint(&spec.target, visit);
        }
        MutationIntent::Create(CreateIntent::BulkRelations(spec)) => {
            for (source, target) in &spec.endpoints {
                endpoint(source, visit);
                endpoint(target, visit);
            }
        }
        MutationIntent::Create(
            CreateIntent::Entity(_)
            | CreateIntent::EntityAspects(_)
            | CreateIntent::BulkEntities(_),
        ) => {}
        MutationIntent::Entity(intent) => {
            let id = match intent {
                EntityMutationIntent::UpdateFields(spec) => spec.entity_id,
                EntityMutationIntent::ApplyAspectPatch(spec) => spec.entity_id,
                EntityMutationIntent::Replace(spec) => spec.entity_id,
                EntityMutationIntent::Delete(spec) => spec.entity_id,
                EntityMutationIntent::Revalidate(spec) => spec.entity_id,
            };
            endpoint(&EntityReference::Existing(id), visit);
        }
        MutationIntent::Relation(intent) => {
            let id = match intent {
                RelationMutationIntent::UpdateEndpoints(spec) => {
                    endpoint(&spec.source, visit);
                    endpoint(&spec.target, visit);
                    spec.relation_id
                }
                RelationMutationIntent::ApplyAspectPatch(spec) => spec.relation_id,
                RelationMutationIntent::Delete(spec) => spec.relation_id,
                RelationMutationIntent::Revalidate(spec) => spec.relation_id,
            };
            visit(RecordRef::Relation(id));
        }
        MutationIntent::Materialization(intent) => visit(intent.record()),
    }
}
