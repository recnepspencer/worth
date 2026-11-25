//! Settle an exact consumed-output prerequisite against its replacement.
use std::sync::Arc;
use worth_execution::ExecutionRequest;
use worth_relational_contracts::internal_port::mvcc::publication::application_publication::output_meaning::{RequiredOutputPublicationMeaning, RequiredOutputRolePosture};
use worth_relational_contracts::internal_port::transactions::data::outcomes::created_entity_bindings::CommitCreatedEntityBindings;
use worth_relational_contracts::internal_port::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;
use worth_relational_mutation::internal_port::mvcc::validation::proposal_identity::RelationalMutationProposalIdentity;
use crate::visibility::materialization::read_records::reader::pinned_read::{RelationalAppliedCandidateReadIdentity, RelationalPinnedRead};
use super::super::inputs::{InputIssuance, InputOrigin, RelationalRequiredOutputInput};
use super::super::retained_assembly::CoupledPublicationPart;
use compare::RoleSignature;

#[path = "settlement/compare.rs"]
pub mod compare;

type CandidatePin = RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>;

/// The exact retained consumer input whose carried meaning may be replaced.
pub struct RetainedInputReplacement {
    consumer: usize,
    retained: usize,
    meaning: RequiredOutputPublicationMeaning,
}
impl RetainedInputReplacement {
    pub fn settle(self, parts: &mut [CoupledPublicationPart]) {
        parts[self.consumer]
            .inputs
            .as_mut()
            .expect("retained consumer inputs")
            .meanings[self.retained] = self.meaning;
    }
}

pub fn compare_retained_input(
    execution: &ExecutionRequest<'_, '_>,
    identity: &RelationalMutationProposalIdentity,
    issuance: Option<&Arc<InputIssuance>>,
    parts: &[CoupledPublicationPart],
    created: &CommitCreatedEntityBindings,
    consumer: &RelationalRequiredOutputInput,
    old_parent_pin: &CandidatePin,
    old_parent: &RelationalRequiredOutputInput,
    replacement: &RelationalRequiredOutputInput,
    pin_current: &mut dyn FnMut() -> Result<CandidatePin, TransactionCommitError>,
) -> Result<Option<RetainedInputReplacement>, TransactionCommitError> {
    let valid = |input: &RelationalRequiredOutputInput| {
        &input.proposal == identity
            && issuance.is_some_and(|issued| Arc::ptr_eq(issued, &input.issuance))
    };
    if !valid(consumer) || !valid(old_parent) || !valid(replacement) {
        return Err(super::super::super::origin_denial(
            "foreign consumed-output settlement proposal",
        ));
    }
    let InputOrigin::Applied(consumer_index) = consumer.origin else {
        return Ok(None);
    };
    let InputOrigin::Applied(rep_index) = replacement.origin else {
        return Ok(None);
    };
    if consumer_index == rep_index {
        return Ok(None);
    }
    let Some(rep_part) = parts.get(rep_index) else {
        return Ok(None);
    };
    let Some(rep_meaning) = rep_part.publication.producing_meaning() else {
        return Ok(None);
    };
    let Some((rep_producer, _)) = rep_part.publication.lineage_attribution() else {
        return Ok(None);
    };
    // The consumer must have retained this route; its retained meaning is
    // the historical one it consumed, kept from before any fold.
    let Some(retained) = parts[consumer_index].inputs.as_ref().and_then(|inputs| {
        inputs.meanings.iter().position(|entry| {
            entry.family == rep_meaning.family
                && entry.source_scope == rep_meaning.source_scope
                && entry.source_partition == rep_meaning.source_partition
        })
    }) else {
        return Ok(None);
    };
    let retained_meaning = &parts[consumer_index]
        .inputs
        .as_ref()
        .expect("retained consumer inputs")
        .meanings[retained];
    let (old_meaning, old_producer, old_roles) = match &old_parent.origin {
        InputOrigin::Applied(head_index) => {
            if *head_index >= consumer_index || *head_index != rep_index {
                return Ok(None);
            }
            // A successor round supersedes its chain in place: the same
            // route and producer (`route::require_same_route`), so the
            // chain's part carries its live round's producer and roles
            // (re-resolved at adoption); the meaning a round may change
            // (implementation, epoch) is the retained one.
            let head = &parts[*head_index];
            let (producer, _) = head.publication.lineage_attribution().expect("head owner");
            let roles = resolve_part_roles(execution, &parts[*head_index], created)?;
            (retained_meaning, producer, roles)
        }
        InputOrigin::Current(publication) => {
            let meaning = &publication.origin.meaning;
            if retained_meaning != meaning {
                return Ok(None);
            }
            let producer = publication.origin.producer.as_str();
            let mut roles = Vec::with_capacity(publication.origin.members.len());
            for member in &publication.origin.members {
                roles.push(RoleSignature {
                    role: member.role.as_str(),
                    entity_name: member.entity_name.as_str(),
                    kind: member.kind,
                    entity: member.id,
                    retiring: member.posture == RequiredOutputRolePosture::Retire,
                });
            }
            (meaning, producer, roles)
        }
        InputOrigin::Restored { .. } => return Ok(None),
    };
    if old_producer != rep_producer
        || old_meaning.family != rep_meaning.family
        || old_meaning.source_scope != rep_meaning.source_scope
        || old_meaning.source_partition != rep_meaning.source_partition
        || old_meaning.implementation != rep_meaning.implementation
    {
        return Ok(None);
    }
    let rep_roles = resolve_part_roles(execution, rep_part, created)?;
    if old_roles != rep_roles {
        return Ok(None);
    }
    let current_pin = pin_current()?;
    // Each pin's read residency lends its own selected partitions.
    let old_root = old_parent_pin.resident_basis().basis();
    let new_root = current_pin.resident_basis().basis();
    let payload_equal = compare::payloads_equal(execution, old_root, new_root, &rep_roles)?;
    if !payload_equal {
        return Ok(None);
    }
    Ok(Some(RetainedInputReplacement {
        consumer: consumer_index,
        retained,
        meaning: rep_meaning.clone(),
    }))
}

fn resolve_part_roles<'a>(
    execution: &ExecutionRequest<'_, '_>,
    part: &'a CoupledPublicationPart,
    created: &CommitCreatedEntityBindings,
) -> Result<Vec<RoleSignature<'a>>, TransactionCommitError> {
    let roles = part.publication.roles();
    let resolved =
        super::super::producer_posture::resolve_applied_roles(execution, roles, created)?;
    Ok(roles
        .iter()
        .zip(resolved.iter().copied())
        .map(|(role, entity)| RoleSignature {
            role: role.role.as_str(),
            entity_name: role.entity_name.as_str(),
            kind: role.kind,
            entity,
            retiring: role.posture == RequiredOutputRolePosture::Retire,
        })
        .collect())
}
