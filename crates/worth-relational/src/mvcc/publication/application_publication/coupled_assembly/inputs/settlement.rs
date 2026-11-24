//! Native half of R3 exact-consumption settlement (PR C §1.2).
//! Compares roles, entity ids, generations, custody, and stored aspect payloads
//! between an old consumed parent and its chain replacement, settling the
//! consumer's `RequiredOutputInputs` only on exact Native equality.
use super::*;
use crate::identity::data::{EntityId, KindId};
use crate::storage::overlay::CurrentPartitionAccess;

type CandidatePin = RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>;

#[derive(Clone, Debug, Eq, PartialEq)]
struct RoleSignature<'a> {
    role: &'a str,
    entity_name: &'a str,
    kind: KindId,
    entity: EntityId,
    retiring: bool,
}

impl RelationalAuthoredCoupledCandidate<'_> {
    /// Compare Native consumption identity and payload between `old_parent` (at
    /// `old_parent_pin`) and `replacement` (at the current candidate pin). If
    /// equal, settle `consumer`'s retained `RequiredOutputInputs` for that route
    /// to `replacement`'s meaning and return `Ok(true)`. Otherwise return
    /// `Ok(false)` without mutating `consumer`.
    pub fn settle_output_input(
        &mut self,
        execution: &ExecutionRequest<'_, '_>,
        consumer: &RelationalRequiredOutputInput,
        old_parent_pin: &CandidatePin,
        old_parent: &RelationalRequiredOutputInput,
        replacement: &RelationalRequiredOutputInput,
    ) -> Result<bool, TransactionCommitError> {
        let issuance = self.input_issuance.borrow().as_ref().cloned();
        let valid = |input: &RelationalRequiredOutputInput| {
            input.proposal == *self.identity
                && issuance
                    .as_ref()
                    .is_some_and(|issued| Arc::ptr_eq(issued, &input.issuance))
        };
        if !valid(consumer) || !valid(old_parent) || !valid(replacement) {
            return Err(super::super::origin_denial(
                "foreign consumed-output settlement proposal",
            ));
        }
        let InputOrigin::Applied(consumer_index) = consumer.origin else {
            return Ok(false);
        };
        let InputOrigin::Applied(rep_index) = replacement.origin else {
            return Ok(false);
        };
        if consumer_index == rep_index {
            return Ok(false);
        }
        let Some(rep_part) = self.parts.get(rep_index) else {
            return Ok(false);
        };
        let Some(rep_meaning) = rep_part.publication.producing_meaning() else {
            return Ok(false);
        };
        let Some((rep_producer, _)) = rep_part.publication.lineage_attribution() else {
            return Ok(false);
        };
        // The consumer must have retained this route; its retained meaning is
        // the historical one it consumed, kept from before any fold.
        let Some(retained) = self.parts[consumer_index]
            .inputs
            .as_ref()
            .and_then(|inputs| {
                inputs.meanings.iter().position(|entry| {
                    entry.family == rep_meaning.family
                        && entry.source_scope == rep_meaning.source_scope
                        && entry.source_partition == rep_meaning.source_partition
                })
            })
        else {
            return Ok(false);
        };
        let retained_meaning = &self.parts[consumer_index]
            .inputs
            .as_ref()
            .expect("retained consumer inputs")
            .meanings[retained];
        let (old_meaning, old_producer, old_roles) = match &old_parent.origin {
            InputOrigin::Applied(head_index) => {
                if *head_index >= consumer_index || *head_index != rep_index {
                    return Ok(false);
                }
                // A fold admits a round only with the head's producer and
                // roles over the same records (`prove_succession` Identity),
                // so the chain's current part carries them; the meaning a
                // fold may change (implementation, epoch) is the retained one.
                let head = &self.parts[*head_index];
                let (producer, _) = head.publication.lineage_attribution().expect("head owner");
                let roles = self.resolve_part_roles(execution, *head_index)?;
                (retained_meaning, producer, roles)
            }
            InputOrigin::Current(publication) => {
                let meaning = &publication.origin.meaning;
                if retained_meaning != meaning {
                    return Ok(false);
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
            InputOrigin::Restored { .. } => return Ok(false),
        };
        if old_producer != rep_producer
            || old_meaning.family != rep_meaning.family
            || old_meaning.source_scope != rep_meaning.source_scope
            || old_meaning.source_partition != rep_meaning.source_partition
            || old_meaning.implementation != rep_meaning.implementation
        {
            return Ok(false);
        }
        let rep_roles = self.resolve_part_roles(execution, rep_index)?;
        if old_roles != rep_roles {
            return Ok(false);
        }
        let current_pin = self.pin_source(execution)?;
        let old_root = old_parent_pin.resident_basis().root();
        let new_root = current_pin.resident_basis().root();
        let payload_equal = execution
            .run_native_with_checkpoint(|context| {
                for role in &rep_roles {
                    checkpoint(context, 1).map_err(custody::execution_stop)?;
                    if !entity_payload_equal(old_root.as_ref(), new_root.as_ref(), role.entity) {
                        return Ok::<_, TransactionCommitError>(false);
                    }
                }
                Ok(true)
            })
            .map_err(custody::execution_stop)?
            .0?;
        if !payload_equal {
            return Ok(false);
        }
        let next_meaning = rep_meaning.clone();
        self.parts[consumer_index]
            .inputs
            .as_mut()
            .expect("retained consumer inputs")
            .meanings[retained] = next_meaning;
        Ok(true)
    }

    fn resolve_part_roles(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        index: usize,
    ) -> Result<Vec<RoleSignature<'_>>, TransactionCommitError> {
        let part = &self.parts[index];
        let roles = part.publication.roles();
        let resolved = super::producer_posture::resolve_applied_roles(
            execution,
            roles,
            &self.mutation.outcome.created_entities,
        )?;
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
}

fn entity_payload_equal(
    old_root: &impl CurrentPartitionAccess,
    new_root: &impl CurrentPartitionAccess,
    entity: EntityId,
) -> bool {
    let slot = entity.slot_index();
    let old_slot = old_root
        .get_current_partition(entity.partition_id)
        .and_then(|p| p.entity_arena().get_slot(slot));
    let new_slot = new_root
        .get_current_partition(entity.partition_id)
        .and_then(|p| p.entity_arena().get_slot(slot));
    let (Some(old_slot), Some(new_slot)) = (old_slot, new_slot) else {
        return false;
    };
    if old_slot.record_id() != entity
        || new_slot.record_id() != entity
        || !old_slot.is_live()
        || !new_slot.is_live()
        || old_slot.kind_id() != new_slot.kind_id()
    {
        return false;
    }
    old_slot.extra().authoritative_aspect_state() == new_slot.extra().authoritative_aspect_state()
}
