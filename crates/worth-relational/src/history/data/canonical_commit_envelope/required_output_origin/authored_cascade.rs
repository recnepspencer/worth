//! One atomic authored retirement retains every genuine route's own provenance.
use super::*;
use crate::{
    history::data::RequiredOutputPublicationPositions,
    mvcc::{RelationalAuthoredRetirement, RequiredOutputPublication},
    storage::overlay::WorkingState,
    transactions::data::{
        CommitCreatedEntityBindings, CommitCreatedRelationBindings, EntityReference,
    },
};
use std::sync::Arc;
use worth_execution::{ExecutionResourceLease, MapKernelContext};

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct CanonicalAuthoredRetirement {
    pub(crate) anchor: EntityId,
    pub(crate) birth: CommitId,
    pub(crate) publications: Vec<Arc<CanonicalRequiredOutputOrigin>>,
    #[serde(skip)]
    pub(crate) _decoded_memory: Option<std::sync::Arc<worth_execution::ExecutionMemoryReservation>>,
}

#[allow(clippy::too_many_arguments)]
pub(super) fn install(
    envelope: &mut CanonicalCommitEnvelopeBody,
    runtime: &crate::runtime::RelationalPreparationRuntime,
    basis: &AdmittedRelationalBranchBasis,
    created: &CommitCreatedEntityBindings,
    created_relations: &CommitCreatedRelationBindings,
    state: &WorkingState,
    installed: crate::mvcc::RelationalInstalledApplicationPublicationAuthority,
    admission: worth_proof::AdmittedAuthoredOutputRetirement,
    mut predecessor: RelationalAuthoredRetirement,
    lease: Option<&ExecutionResourceLease<'_>>,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<RequiredOutputPublicationPositions, TransactionCommitError> {
    use crate::{
        publication::patch::data::RecordStructuralChange as Change, transactions::data::RecordRef,
    };
    let roles = std::mem::take(&mut predecessor.roles);
    let anchor = predecessor.anchor;
    let refuse = || {
        TransactionCommitError::publication(
            crate::publication::data::PublicationError::authored_retirement(
                crate::publication::data::AuthoredRetirementRefusal::InvalidAuthoredAnchor,
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
    use crate::publication::data::{AuthoredRetirementRefusal as Cause, PublicationError};
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
    envelope.authored_cascade_retirement = Some(Arc::new(CanonicalAuthoredRetirement {
        _decoded_memory: None,
        anchor: predecessor.anchor,
        birth: predecessor.birth,
        publications: Vec::new(),
    }));
    let mut positions = basis.inner.root.output_publication_positions.clone();
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
            .filter(|member| member.posture != crate::mvcc::RequiredOutputRolePosture::Retire)
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
        envelope
            .install_completion(
                runtime,
                basis,
                created,
                created_relations,
                state,
                crate::mvcc::RequiredOutputPublicationAdmission {
                    installed: installed.clone(),
                    publication: RequiredOutputPublication::AuthoredRetirement {
                        admission: single,
                        predecessor: prior,
                        roles: members,
                    },
                },
                lease,
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
        let retired = envelope.required_output_origin.take().ok_or_else(refuse)?;
        positions.insert(
            &retired.meaning,
            crate::history::data::RequiredOutputRoutePosition::Published(envelope.commit.commit_id),
            lease,
            context,
        )?;
        publications.push(retired);
    }
    envelope.authored_cascade_retirement = Some(Arc::new(CanonicalAuthoredRetirement {
        _decoded_memory: None,
        anchor: predecessor.anchor,
        birth: predecessor.birth,
        publications,
    }));
    Ok(positions)
}

impl PartialEq for CanonicalAuthoredRetirement {
    fn eq(&self, other: &Self) -> bool {
        self.anchor == other.anchor
            && self.birth == other.birth
            && self.publications == other.publications
    }
}
impl Eq for CanonicalAuthoredRetirement {}
