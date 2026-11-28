//! Admit complete dynamic membership from native allocations and prior live rows.
use super::*;
use contracts::mvcc::publication::application_publication::output_meaning::RequiredOutputRolePosture as Posture;
use contracts::mvcc::publication::application_publication::output_meaning::RequiredOutputRoleTarget;
use contracts::publication::patch::data::record_structural_change::RecordStructuralChange;
use contracts::transactions::data::outcomes::created_entity_bindings::CommitCreatedEntityBindings;
use contracts::transactions::data::primitives::EntityReference;
use contracts::transactions::data::primitives::RecordRef;
use worth_execution::MapKernelContext;
use worth_relational_branch::internal_port::branch::basis::{
    AdmittedRelationalBranchBasis, AdmittedRelationalBranchBasisFieldsPort as _,
};
use worth_relational_contracts::internal_port as contracts;
use worth_relational_storage::internal_port::storage::overlay::access::OverlayStateView;
use worth_relational_storage::internal_port::storage::overlay::current_access::CurrentPartitionAccess;
use worth_relational_storage::internal_port::storage::overlay::slot_resolution::current_entity_partition_source_for_slot;
use worth_relational_storage::internal_port::storage::overlay::working_state::WorkingState;

#[allow(clippy::too_many_arguments)]
pub(super) fn capture(
    envelope: &CanonicalCommitEnvelopeBody,
    history: &crate::publication::ownership::required_output_origin::history_request::PreparationHistoryRequest<'_, '_, '_>,
    basis: &AdmittedRelationalBranchBasis,
    state: &WorkingState,
    created: &CommitCreatedEntityBindings,
    roles: &[RequiredOutputRoleTarget],
    previous: Option<&CanonicalRequiredOutputOrigin>,
    producer: &str,
    dependency: [u8; 32],
    context: &mut MapKernelContext<'_, '_>,
) -> Result<Vec<CanonicalOutputMember>, TransactionCommitError> {
    let after_image = OverlayStateView::new(basis, state);
    let mut members = Vec::with_capacity(roles.len());
    for (member_ordinal, role) in roles.iter().enumerate() {
        context
            .checkpoint((role.role.len() + role.entity_name.len() + 2) as u64)
            .map_err(origin_request)?;
        let (id, old, before_write) = match (&role.target, role.posture) {
            (EntityReference::Created(target), Posture::Create) => {
                context
                    .checkpoint(created.resolve_work_bound(target).ok_or_else(overflow)?)
                    .map_err(origin_request)?;
                (
                    created.resolve(target).ok_or_else(|| {
                        contracts::transactions::data::outcomes::transaction_commit_error::origin_denial("output was not allocated by this mutation")
                    })?,
                    None,
                    None,
                )
            }
            (EntityReference::Existing(id), Posture::Preserve | Posture::Retire) => {
                let prior = previous.ok_or_else(|| {
                    contracts::transactions::data::outcomes::transaction_commit_error::origin_denial(
                        "existing output has no complete predecessor publication",
                    )
                })?;
                context
                    .checkpoint(u64::from(usize::BITS - prior.members.len().leading_zeros()) + 1)
                    .map_err(origin_request)?;
                let member = prior
                    .members
                    .binary_search_by_key(id, |member| member.id)
                    .ok()
                    .map(|index| &prior.members[index])
                    .filter(|member| member.posture != Posture::Retire)
                    .ok_or_else(|| {
                        contracts::transactions::data::outcomes::transaction_commit_error::origin_denial(
                            "existing target is outside the prior live publication",
                        )
                    })?;
                let partition = basis

                    .get_current_partition(id.partition_id)
                    .ok_or_else(|| contracts::transactions::data::outcomes::transaction_commit_error::origin_denial("prior output partition absent"))?;
                let row = partition
                    .entity_arena()
                    .get(id)
                    .filter(|row| row.is_live())
                    .ok_or_else(|| {
                        contracts::transactions::data::outcomes::transaction_commit_error::origin_denial("prior output generation absent or retired")
                    })?;
                let write = partition
                    .entity_arena()
                    .current_metadata_at(id.slot_index())
                    .ok_or_else(|| {
                        contracts::transactions::data::outcomes::transaction_commit_error::origin_denial("prior output write position absent")
                    })?
                    .effective_at;
                if member.kind != role.kind
                    || row.kind_id() != Some(role.kind)
                    || member.entity_name != role.entity_name
                    || (write != member.after_write
                        && !worth_relational_branch::internal_port::history::data::codec_retirement_closure::materialization::write_matches(
                            &RecordRef::Entity(*id),
                            prior.commit.commit_id,
                            member.after_write,
                            write,
                            &mut |commit| history.canonical_envelope(commit),
                            &mut |version, context| {
                                history.canonical_envelope_for_version(version, context)
                            },
                            history.canonical_catalog_len(),
                            &history.execution(),
                            context,
                        )
                        .map_err(worth_relational_branch::internal_port::history::data::canonical_commit_envelope::required_output_origin::installation::relation_failure::relation_stop)?)
                {
                    return Err(contracts::transactions::data::outcomes::transaction_commit_error::origin_denial(
                        "intervening native write revoked producer ownership before Preserve",
                    ));
                }
                (*id, Some(member), Some(write))
            }
            _ => return Err(
                contracts::transactions::data::outcomes::transaction_commit_error::origin_denial(
                    "invalid native output posture/target",
                ),
            ),
        };
        let (_, partition) = current_entity_partition_source_for_slot(
            &after_image,
            id.partition_id,
            id.slot_index(),
        )
        .ok_or_else(|| {
            contracts::transactions::data::outcomes::transaction_commit_error::origin_denial(
                "output partition absent",
            )
        })?;
        let record = partition.entity_arena().get(&id).ok_or_else(|| {
            contracts::transactions::data::outcomes::transaction_commit_error::origin_denial(
                "output generation absent",
            )
        })?;
        if record.kind_id() != Some(role.kind)
            || record.is_live() != (role.posture != Posture::Retire)
        {
            return Err(
                contracts::transactions::data::outcomes::transaction_commit_error::origin_denial(
                    "output kind/lifecycle differs from completed role",
                ),
            );
        }
        let after_write = if role.posture == Posture::Retire {
            context
                .checkpoint(envelope.patch.authoritative_record_patches.len() as u64)
                .map_err(origin_request)?;
            if !envelope
                .patch
                .authoritative_record_patches
                .iter()
                .any(|patch| {
                    patch.target == RecordRef::Entity(id)
                        && patch.structural_change == RecordStructuralChange::Deleted
                })
                || record.retired_at() != Some(envelope.commit.version_id)
            {
                return Err(
                    contracts::transactions::data::outcomes::transaction_commit_error::origin_denial(
                        "Retire lacks this publication's actual native deletion",
                    ),
                );
            }
            envelope.commit.version_id
        } else {
            partition
                .entity_arena()
                .current_metadata_at(id.slot_index())
                .ok_or_else(|| {
                    contracts::transactions::data::outcomes::transaction_commit_error::origin_denial(
                        "output write position absent",
                    )
                })?
                .effective_at
        };
        if role.posture == Posture::Preserve {
            context
                .checkpoint(envelope.patch.authoritative_record_patches.len() as u64 + 1)
                .map_err(origin_request)?;
            let patch = envelope
                .patch
                .authoritative_record_patches
                .iter()
                .find(|patch| patch.target == RecordRef::Entity(id));
            if patch.is_some_and(|patch| {
                patch.structural_change != RecordStructuralChange::Updated
                    || patch.contains_opaque_aspect
            }) || (before_write.is_some_and(|before| before != after_write)
                && after_write != envelope.commit.version_id)
            {
                return Err(
                    contracts::transactions::data::outcomes::transaction_commit_error::origin_denial(
                        "Preserve differs from its actual supported native mutation",
                    ),
                );
            }
        }
        if role.posture == Posture::Create {
            let schema = basis.owner_inner().root.schema_authority();
            context
                .checkpoint(
                    u64::from(usize::BITS - schema.entity_aspect_plan_count().leading_zeros()) + 1,
                )
                .map_err(origin_request)?;
            let plan = schema.entity_aspect_plan(role.kind).ok_or_else(|| {
                contracts::transactions::data::outcomes::transaction_commit_error::origin_denial(
                    "output physical contract plan absent",
                )
            })?;
            worth_relational_branch::internal_port::history::data::canonical_commit_envelope::required_output_origin::readmission::validate_created_output_for_commit(
                envelope,
                id,
                plan,
                record.extra().authoritative_aspect_state(),
                member_ordinal,
                context,
            )?;
        }
        members.push(CanonicalOutputMember {
            role: role.role.clone(),
            entity_name: role.entity_name.clone(),
            id,
            kind: role.kind,
            posture: role.posture,
            birth: old.map_or(envelope.commit.commit_id, |member| member.birth),
            birth_producer: old.map_or_else(
                || producer.to_owned(),
                |member| member.birth_producer.clone(),
            ),
            birth_dependency: old.map_or(
                worth_foundational::CanonicalDigestId::new(dependency),
                |member| member.birth_dependency,
            ),
            before_write,
            after_write,
        });
    }
    let levels = (usize::BITS - members.len().leading_zeros()) as usize + 1;
    context
        .checkpoint(
            (members
                .len()
                .checked_mul(levels)
                .and_then(|n| n.checked_mul(2))
                .ok_or_else(overflow)?) as u64,
        )
        .map_err(origin_request)?;
    members.sort_unstable_by(|left, right| left.role.cmp(&right.role));
    if members.windows(2).any(|pair| pair[0].role == pair[1].role) {
        return Err(
            contracts::transactions::data::outcomes::transaction_commit_error::origin_denial(
                "publication repeats output role",
            ),
        );
    }
    members.sort_unstable_by_key(|member| member.id);
    if members.windows(2).any(|pair| pair[0].id == pair[1].id) {
        return Err(
            contracts::transactions::data::outcomes::transaction_commit_error::origin_denial(
                "publication repeats native output",
            ),
        );
    }
    if let Some(previous) = previous {
        for old in &previous.members {
            context
                .checkpoint(levels as u64 + 1)
                .map_err(origin_request)?;
            if old.posture != Posture::Retire
                && members
                    .binary_search_by_key(&old.id, |member| member.id)
                    .is_err()
            {
                return Err(
                    contracts::transactions::data::outcomes::transaction_commit_error::origin_denial(
                        "successor omits prior live output member",
                    ),
                );
            }
        }
    }
    Ok(members)
}

fn overflow() -> TransactionCommitError {
    origin_request(
        contracts::execution::denial_cause::RelationalExecutionDenialCause::WorkCounterOverflow,
    )
}
