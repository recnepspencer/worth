//! Atomic replacement of carrier/target case groups through Native metadata intents.
use super::*;
use crate::aggregate_identity::funding::PlanDenial;
use crate::aggregate_identity::membership::key_lookup;
use crate::aggregate_identity::projected_storage::{
    domain_for_contract, encode_cases, MEMBERSHIP_ASPECT,
};
use crate::aggregate_identity::transition::AggregateTransition;
use crate::transactions::data::ApplyRelationAspectPatchIntent;
use worth_foundational::facade::{
    AspectKey, AspectValue, ContractValidationInput, FieldKey, PortableAspectContractBasis,
    PortableAspectPatchOperation, PortableRecordAspectPatch, StructAspectValue,
};

type GroupKey = (EntityId, AggregateIdentity);
type Groups = BTreeMap<GroupKey, BTreeSet<Member>>;

pub(super) fn append(
    plan: &mut ConsequencePlan,
    registry: &RelationalSchemaRegistry,
    contract: &AggregateIdentityContractDeclaration,
    targets: &BTreeMap<AggregateIdentity, EntityReference>,
    prior: &BTreeMap<Member, (RelationId, AggregateIdentity)>,
    assignments: &BTreeMap<Member, usize>,
    transition: &AggregateTransition,
    after: &InvariantStateView<'_>,
    work: &mut AggregateWork<'_, '_, '_, '_>,
) -> BuildResult<()> {
    let mut old = Groups::new();
    let mut old_edges = BTreeMap::new();
    for (member, &(edge, identity)) in prior {
        let key = (member.carrier(), identity);
        group(&mut old, key, member, work)?;
        if old_edges
            .insert(key, edge)
            .is_some_and(|previous| previous != edge)
        {
            return work.invalid(PlanDenial::IncompleteStoredMembership);
        }
    }
    let mut desired = Groups::new();
    for (member, &assignment) in assignments {
        let identity = transition.assignments[assignment].identity;
        group(&mut desired, (member.carrier(), identity), member, work)?;
    }
    let domain = domain_for_contract(registry, contract, work)?;
    for (&key, members) in &old {
        work.checkpoint(1)?;
        let relation = old_edges[&key];
        if let Some(next) = desired.get(&key) {
            for member in members.iter().chain(next) {
                key_lookup(member, 1, work)?;
            }
            if members == next {
                continue;
            }
            let Some(domain) = &domain else {
                return work.invalid(PlanDenial::IncompleteStoredMembership);
            };
            let text = encode_group(domain, key.0, next, work)?;
            update(plan, registry, contract, relation, key, targets, text, work)?;
        } else if after.relation_metadata(relation).is_some() {
            // A carrier deletion can have already retired its private edges.
            // Never restore them from the retained before image.
            plan.demands.relations.insert(relation);
            plan.demands.partitions.insert(relation.partition_id);
            plan.endpoint(&EntityReference::Existing(key.0));
            plan.endpoint(&targets[&key.1]);
            plan.push(
                MutationIntent::Relation(RelationMutationIntent::Delete(DeleteRelationIntent {
                    relation_id: relation,
                })),
                work,
            )?;
        }
    }
    for (&key, members) in &desired {
        work.checkpoint(1)?;
        if old.contains_key(&key) {
            continue;
        }
        work.retain(extent::membership_mutation())?;
        let source = EntityReference::Existing(key.0);
        let target = targets[&key.1].clone();
        let (client_key, fields) = if let Some(domain) = &domain {
            let encoded = encode_group(domain, key.0, members, work)?;
            let fields = BTreeMap::from([(
                super::super::record::locator(MEMBERSHIP_ASPECT, "cases"),
                AspectValue::String(encoded.into()),
            )]);
            (
                format!(
                    "aggregate-projected-member:{}:{}:{}:{}:{}",
                    contract.membership_kind.0,
                    key.0.partition_value(),
                    key.0.local_slot_value(),
                    key.0.generation_value(),
                    identity_key(key.1)
                ),
                AspectFieldPatch::new(fields),
            )
        } else {
            if members.len() != 1 || members.first().and_then(Member::entity) != Some(key.0) {
                return work.invalid(PlanDenial::IncompleteStoredMembership);
            }
            (
                format!(
                    "aggregate-member:{}:{}:{}:{}",
                    contract.membership_kind.0,
                    key.0.partition_value(),
                    key.0.local_slot_value(),
                    key.0.generation_value()
                ),
                Default::default(),
            )
        };
        plan.endpoint(&source);
        plan.endpoint(&target);
        plan.push(
            MutationIntent::Create(CreateIntent::Relation(RelationSpec {
                partition_id: key.0.partition_id,
                kind_id: contract.membership_kind,
                client_key: ClientKey::raw(client_key),
                source,
                target,
                fields,
            })),
            work,
        )?;
    }
    Ok(())
}

fn group(
    groups: &mut Groups,
    key: GroupKey,
    member: &Member,
    work: &mut AggregateWork<'_, '_, '_, '_>,
) -> BuildResult<()> {
    work.checkpoint(1)?;
    work.retain(
        extent::tree_entry::<GroupKey, BTreeSet<Member>>() + extent::tree_entry::<Member, ()>(),
    )?;
    let members = groups.entry(key).or_default();
    key_lookup(member, members.len(), work)?;
    if !members.insert(member.clone()) {
        return work.invalid(PlanDenial::IncompleteStoredMembership);
    }
    Ok(())
}
fn encode_group(
    domain: &crate::projected_member_domain::ProjectedMemberDomainHandle,
    carrier: EntityId,
    members: &BTreeSet<Member>,
    work: &mut AggregateWork<'_, '_, '_, '_>,
) -> BuildResult<String> {
    work.retain(
        (members.len() as u64)
            .checked_mul(extent::vector_entry::<&str>())
            .ok_or(worth_execution::MapKernelFailure::ResultCapacityExceeded)?,
    )?;
    let mut cases = Vec::new();
    for member in members {
        let Member::Projected(member) = member else {
            return work.invalid(PlanDenial::IncompleteStoredMembership);
        };
        work.checkpoint(
            member
                .domain
                .name_bytes()
                .and_then(|n| u64::try_from(n).ok())
                .ok_or(worth_execution::MapKernelFailure::ResultCapacityExceeded)?,
        )?;
        if member.carrier != carrier || member.domain != domain.parts().identity {
            return work.invalid(PlanDenial::IncompleteStoredMembership);
        }
        cases.push(member.case.as_str());
    }
    // Every key shares this exact domain/carrier, so semantic key order is the
    // canonical case order; the sole codec still validates the real roster.
    encode_cases(domain, &cases, work)
}
fn update(
    plan: &mut ConsequencePlan,
    registry: &RelationalSchemaRegistry,
    contract: &AggregateIdentityContractDeclaration,
    relation: RelationId,
    key: GroupKey,
    targets: &BTreeMap<AggregateIdentity, EntityReference>,
    text: String,
    work: &mut AggregateWork<'_, '_, '_, '_>,
) -> BuildResult<()> {
    let registered = registry
        .relation_registration(contract.membership_kind)
        .map_err(|_| {
            worth_execution::MapKernelFailure::Domain(PlanDenial::DefinitionUnavailable)
        })?;
    let aspect_key = AspectKey::new(MEMBERSHIP_ASPECT).expect("fixed membership aspect");
    let Some(binding) = registered
        .aspect_contract_declarations
        .aspects
        .iter()
        .find(|binding| binding.contract.key() == &aspect_key)
    else {
        return work.invalid(PlanDenial::DefinitionUnavailable);
    };
    work.retain(extent::scalar_fields(1))?;
    let value = StructAspectValue::new([(
        FieldKey::new("cases").expect("fixed case field"),
        AspectValue::String(text.into()),
    )])
    .map_err(|_| worth_execution::MapKernelFailure::Domain(PlanDenial::InvalidStoredIdentity))?;
    let aspect_patch = PortableRecordAspectPatch::new([PortableAspectPatchOperation::SetWhole {
        basis: PortableAspectContractBasis::from_contract(&binding.contract),
        value: ContractValidationInput::Struct(value),
    }]);
    plan.demands.relations.insert(relation);
    plan.demands.partitions.insert(relation.partition_id);
    plan.endpoint(&EntityReference::Existing(key.0));
    plan.endpoint(&targets[&key.1]);
    plan.push(
        MutationIntent::Relation(RelationMutationIntent::ApplyAspectPatch(
            ApplyRelationAspectPatchIntent {
                relation_id: relation,
                aspect_patch,
            },
        )),
        work,
    )
}
