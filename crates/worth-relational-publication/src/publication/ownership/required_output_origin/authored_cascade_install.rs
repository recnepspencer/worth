//! Install the selected Native authored retirement, including its producer completions.
use super::*;
use crate::mvcc::publication::application_publication::RelationalAuthoredRetirement;
use crate::mvcc::publication::application_publication::RequiredOutputPublication;
use contracts::publication::data::publication_error::PublicationErrorOwnerPort as _;
use contracts::transactions::data::outcomes::created_entity_bindings::CommitCreatedEntityBindings;
use contracts::transactions::data::outcomes::created_relation_bindings::CommitCreatedRelationBindings;
use contracts::transactions::data::primitives::EntityReference;
use std::sync::Arc;
use worth_execution::{ExecutionResourceLease, MapKernelContext};
use worth_relational_branch::internal_port::branch::basis::{
    AdmittedRelationalBranchBasis, AdmittedRelationalBranchBasisFieldsPort as _,
};
use worth_relational_branch::internal_port::history::data::canonical_commit_envelope::CanonicalCommitEnvelopeBodyFieldsPort as _;
use worth_relational_branch::internal_port::history::data::required_output_positions::RequiredOutputPublicationPositions;
use worth_relational_contracts::internal_port as contracts;
use worth_relational_storage::internal_port::storage::overlay::working_state::WorkingState;

use worth_relational_branch::internal_port::history::data::canonical_commit_envelope::required_output_origin::authored_cascade::CanonicalAuthoredRetirement;

#[allow(clippy::too_many_arguments)]
pub(super) fn install(
    envelope: &mut CanonicalCommitEnvelopeBody,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &crate::publication::ownership::required_output_origin::history_request::OutputPreparationRequest<'_, '_, '_>,
    basis: &AdmittedRelationalBranchBasis,
    created: &CommitCreatedEntityBindings,
    created_relations: &CommitCreatedRelationBindings,
    state: &WorkingState,
    installed: crate::mvcc::publication::application_publication::RelationalInstalledApplicationPublicationAuthority,
    admission: &worth_proof::AdmittedAuthoredOutputRetirement,
    mut predecessor: RelationalAuthoredRetirement,
    lease: Option<&ExecutionResourceLease<'_>>,
    validated_history: &mut OutputOriginHistory,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<RequiredOutputPublicationPositions, TransactionCommitError> {
    use contracts::publication::patch::data::record_structural_change::RecordStructuralChange as Change;
    use contracts::transactions::data::primitives::RecordRef;
    let roles = std::mem::take(&mut predecessor.roles);
    let anchor = predecessor.anchor;
    let refuse = || {
        TransactionCommitError::publication(
            contracts::publication::data::publication_error::PublicationError::authored_retirement(
                contracts::publication::data::authored_retirement_refusal::AuthoredRetirementRefusal::InvalidAuthoredAnchor,
                RecordRef::Entity(anchor),
            ),
        )
    };
    context
        .checkpoint(
            roles
                .iter()
                .map(|role| role.role.len() + role.entity_name.len() + 8)
                .sum::<usize>() as u64,
        )
        .map_err(origin_request)?;
    if predecessor.before.descriptor() != basis.descriptor()
        || !envelope
            .patch
            .authoritative_record_patches
            .iter()
            .any(|patch| {
                patch.target == RecordRef::Entity(predecessor.anchor)
                    && patch.structural_change == Change::Deleted
            })
        || admission.publication_count() != predecessor.publications.len()
    {
        return Err(refuse());
    }
    for &(id, _) in &predecessor.co_anchors {
        context
            .checkpoint(envelope.patch.authoritative_record_patches.len() as u64 + 1)
            .map_err(origin_request)?;
        if !envelope
            .patch
            .authoritative_record_patches
            .iter()
            .any(|patch| {
                patch.target == RecordRef::Entity(id) && patch.structural_change == Change::Deleted
            })
        {
            return Err(TransactionCommitError::publication(
                contracts::publication::data::publication_error::PublicationError::authored_retirement(
                    contracts::publication::data::authored_retirement_refusal::AuthoredRetirementRefusal::MissingMember,
                    RecordRef::Entity(id),
                ),
            ));
        }
    }
    use worth_relational_branch::internal_port::publication::data::{
        AuthoredRetirementRefusal as Cause, PublicationError,
    };
    for patch in &envelope.patch.authoritative_record_patches {
        context
            .checkpoint(roles.len() as u64 + 1)
            .map_err(origin_request)?;
        if let RecordRef::Entity(id) = patch.target
            && patch.structural_change == Change::Deleted
            && !roles
                .iter()
                .any(|role| role.target == EntityReference::Existing(id))
        {
            return Err(TransactionCommitError::publication(
                PublicationError::authored_retirement(Cause::ExtraEntity, patch.target.clone()),
            ));
        }
    }
    // Only this private installer, after consuming Native's sealed cascade,
    // establishes the enclosing retirement event used by per-route installation.
    (*envelope.owner_authored_cascade_retirement_mut()) =
        Some(Arc::new(CanonicalAuthoredRetirement {
            _decoded_memory: None,
            anchor: predecessor.anchor,
            owned_members: predecessor.owned_members,
            birth: predecessor.birth,
            co_anchors: predecessor.co_anchors.clone(),
            publications: Vec::new(),
        }));
    let mut positions = basis
        .owner_inner()
        .root
        .output_publication_positions
        .clone();
    let mut publications = Vec::with_capacity(predecessor.publications.len());
    for prior in predecessor.publications {
        let origin = &prior.origin;
        context
            .checkpoint((roles.len().saturating_mul(origin.members.len() + 1)) as u64)
            .map_err(origin_request)?;
        let members = roles
            .iter()
            .filter(|role| match role.target {
                EntityReference::Existing(id) => origin.outputs.binary_search(&id).is_ok(),
                _ => false,
            })
            .cloned()
            .collect();
        for record in origin
            .members
            .iter()
            .filter(|member| member.posture != contracts::mvcc::publication::application_publication::output_meaning::RequiredOutputRolePosture::Retire)
            .map(|member| RecordRef::Entity(member.id))
            .chain(
                origin
                    .relations
                    .iter()
                    .map(|edge| RecordRef::Relation(edge.id)),
            )
        {
            context
                .checkpoint(envelope.patch.authoritative_record_patches.len() as u64 + 1)
                .map_err(origin_request)?;
            if !envelope
                .patch
                .authoritative_record_patches
                .iter()
                .any(|patch| patch.target == record && patch.structural_change == Change::Deleted)
            {
                return Err(TransactionCommitError::publication(
                    PublicationError::authored_retirement(Cause::MissingMember, record),
                ));
            }
        }
        let single = admission
            .for_publication(prior.authority())
            .ok_or_else(refuse)?;
        super::installation::complete::install_completion(
            envelope,
            execution,
            runtime,
            basis,
            created,
            created_relations,
            state,
            crate::mvcc::publication::application_publication::RequiredOutputPublicationAdmission {
                inputs: Default::default(),
                installed: installed.clone(),
                publication: RequiredOutputPublication::AuthoredRetirement {
                    admission: single,
                    predecessor: prior,
                    roles: members,
                },
            },
            None,
            None,
            None,
            lease,
            validated_history,
            context,
        )
        .map_err(|failure| match failure {
            TransactionCommitError::Publication { error, commit_log }
                if error.authored_retirement_refusal().is_none() =>
            {
                let subject = error
                    .context
                    .affected_records
                    .first()
                    .cloned()
                    .unwrap_or(RecordRef::Entity(anchor));
                TransactionCommitError::Publication {
                    error: PublicationError::authored_retirement(
                        Cause::CorruptPublication,
                        subject,
                    ),
                    commit_log,
                }
            }
            other => other,
        })?;
        let retired = (*envelope.owner_required_output_origin_mut())
            .take()
            .ok_or_else(refuse)?;
        positions.insert(
            &retired.meaning,
            worth_relational_branch::internal_port::history::data::required_output_positions::RequiredOutputRoutePosition::Published(envelope.commit.commit_id),
            lease,
            context,
        )?;
        publications.push(retired);
    }
    (*envelope.owner_authored_cascade_retirement_mut()) =
        Some(Arc::new(CanonicalAuthoredRetirement {
            _decoded_memory: None,
            anchor: predecessor.anchor,
            owned_members: predecessor.owned_members,
            birth: predecessor.birth,
            co_anchors: predecessor.co_anchors,
            publications,
        }));
    Ok(positions)
}
