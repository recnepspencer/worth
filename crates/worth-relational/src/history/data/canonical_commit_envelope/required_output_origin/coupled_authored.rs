//! Canonical coupled commits retain each genuine producer route and provenance.
use super::*;
use crate::{
    history::data::{RequiredOutputPublicationPositions, RequiredOutputRoutePosition},
    mvcc::{RelationalAuthoredRetirement, RelationalCoupledAuthoredPublication},
    storage::overlay::WorkingState,
    transactions::data::{CommitCreatedEntityBindings, CommitCreatedRelationBindings, RecordRef},
};
use std::sync::Arc;
use worth_execution::{ExecutionResourceLease, MapKernelContext};

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum CanonicalCoupledAuthoredSources {
    Retirement(Arc<CanonicalAuthoredRetirement>),
    Restoration(Arc<CanonicalAuthoredRestoration>),
}
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct CanonicalCoupledParticipant {
    pub(crate) origin: Arc<CanonicalRequiredOutputOrigin>,
    pub(crate) retirement: Option<Arc<CanonicalProducerRetirement>>,
}
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct CanonicalCoupledAuthoredPublication {
    pub(crate) sources: CanonicalCoupledAuthoredSources,
    pub(crate) participants: Vec<CanonicalCoupledParticipant>,
}
impl CanonicalCoupledAuthoredPublication {
    pub(crate) fn output_origins(
        &self,
    ) -> impl Iterator<Item = &Arc<CanonicalRequiredOutputOrigin>> {
        let sources = match &self.sources {
            CanonicalCoupledAuthoredSources::Retirement(event) => &event.publications,
            CanonicalCoupledAuthoredSources::Restoration(event) => &event.publications,
        };
        sources
            .iter()
            .chain(self.participants.iter().flat_map(|part| {
                std::iter::once(&part.origin).chain(
                    part.retirement
                        .iter()
                        .flat_map(|event| event.publications.iter()),
                )
            }))
    }
}

impl CanonicalCommitEnvelope {
    pub(crate) fn authored_source_retirement(&self) -> Option<&Arc<CanonicalAuthoredRetirement>> {
        self.authored_cascade_retirement.as_ref().or_else(|| {
            match &self.coupled_authored_publication.as_ref()?.sources {
                CanonicalCoupledAuthoredSources::Retirement(event) => Some(event),
                CanonicalCoupledAuthoredSources::Restoration(_) => None,
            }
        })
    }
    pub(crate) fn authored_source_restoration(&self) -> Option<&Arc<CanonicalAuthoredRestoration>> {
        self.authored_cascade_restoration.as_ref().or_else(|| {
            match &self.coupled_authored_publication.as_ref()?.sources {
                CanonicalCoupledAuthoredSources::Restoration(event) => Some(event),
                CanonicalCoupledAuthoredSources::Retirement(_) => None,
            }
        })
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn install(
    envelope: &mut CanonicalCommitEnvelope,
    runtime: &crate::runtime::RelationalPreparationRuntime,
    basis: &AdmittedRelationalBranchBasis,
    created: &CommitCreatedEntityBindings,
    created_relations: &CommitCreatedRelationBindings,
    state: &WorkingState,
    installed: crate::mvcc::RelationalInstalledApplicationPublicationAuthority,
    group: RelationalCoupledAuthoredPublication,
    lease: Option<&ExecutionResourceLease<'_>>,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<RequiredOutputPublicationPositions, TransactionCommitError> {
    if group.parts.is_empty() {
        // All declared roots were already members of the admitted retirement
        // closure. Use its existing canonical event, with unchanged bytes.
        let custody = &group.binding.owners;
        return if let Some(admission) = &group.binding.retirement_admission {
            super::authored_cascade::install(
                envelope,
                runtime,
                basis,
                created,
                created_relations,
                state,
                installed,
                admission,
                RelationalAuthoredRetirement {
                    roles: custody.roles.clone(),
                    anchor: custody.anchor,
                    birth: custody.birth,
                    co_anchors: custody.co_anchors.clone(),
                    before: custody.before.clone(),
                    publications: custody.retirements.clone(),
                },
                lease,
                context,
            )
        } else {
            Ok(basis.inner.root.output_publication_positions.clone())
        };
    }
    let full = std::mem::take(&mut envelope.patch.authoritative_record_patches);
    let answer: Result<RequiredOutputPublicationPositions, TransactionCommitError> = (|| {
        restrict(
            envelope,
            &full,
            &group.authored_records,
            state,
            basis.inner.root.schema_authority().registry(),
            context,
        )?;
        let custody = &group.binding.owners;
        let sources = if let Some(restoration) = &custody.restoration {
            super::authored_restoration::install(
                envelope,
                runtime,
                basis,
                created,
                created_relations,
                state,
                installed.clone(),
                group.binding.retirement_admission.as_deref(),
                restoration,
                lease,
                context,
            )?;
            CanonicalCoupledAuthoredSources::Restoration(
                envelope
                    .authored_cascade_restoration
                    .take()
                    .expect("Native installed authored source restoration"),
            )
        } else {
            if let Some(admission) = &group.binding.retirement_admission {
                super::authored_cascade::install(
                    envelope,
                    runtime,
                    basis,
                    created,
                    created_relations,
                    state,
                    installed.clone(),
                    admission,
                    RelationalAuthoredRetirement {
                        roles: custody.roles.clone(),
                        anchor: custody.anchor,
                        birth: custody.birth,
                        co_anchors: custody.co_anchors.clone(),
                        before: custody.before.clone(),
                        publications: custody.retirements.clone(),
                    },
                    lease,
                    context,
                )?;
            } else {
                require_authored_sources(envelope, custody, context)?;
                envelope.authored_cascade_retirement =
                    Some(Arc::new(CanonicalAuthoredRetirement {
                        anchor: custody.anchor,
                        birth: custody.birth,
                        co_anchors: custody.co_anchors.clone(),
                        publications: Vec::new(),
                    }));
            }
            CanonicalCoupledAuthoredSources::Retirement(
                envelope
                    .authored_cascade_retirement
                    .take()
                    .expect("Native installed authored source retirement"),
            )
        };
        let mut participants = Vec::with_capacity(group.parts.len());
        for part in group.parts {
            restrict(
                envelope,
                &full,
                &part.records,
                state,
                basis.inner.root.schema_authority().registry(),
                context,
            )?;
            envelope.install_completion(
                runtime,
                basis,
                created,
                created_relations,
                state,
                crate::mvcc::RequiredOutputPublicationAdmission {
                    installed: installed.clone(),
                    publication: part.publication,
                },
                Some(&part.records),
                lease,
                context,
            )?;
            participants.push(CanonicalCoupledParticipant {
                origin: envelope
                    .required_output_origin
                    .take()
                    .expect("Native installed coupled successor"),
                retirement: envelope.producer_cascade_retirement.take(),
            });
        }
        let event = CanonicalCoupledAuthoredPublication {
            sources,
            participants,
        };
        let mut positions = basis.inner.root.output_publication_positions.clone();
        for origin in event.output_origins() {
            positions.insert(
                &origin.meaning,
                RequiredOutputRoutePosition::Published(envelope.commit.commit_id),
                lease,
                context,
            )?;
        }
        envelope.coupled_authored_publication = Some(Arc::new(event));
        Ok(positions)
    })();
    envelope.patch.authoritative_record_patches = full;
    let positions = answer?;
    // Incidence sees the complete canonical birth frame, after every writer's
    // own roster is installed. A partial writer view cannot classify a later
    // writer's edge to an earlier output.
    for origin in envelope.output_origins() {
        crate::history::data::publication_relations::validate(
            &basis.inner.root,
            Some(state),
            origin,
            Some(envelope),
            &mut |commit| runtime.history.canonical_envelope(commit),
            &mut |version| runtime.history.canonical_envelope_for_version(version),
            runtime.history.canonical_catalog_len(),
            lease,
            context,
        )
        .map_err(super::installation::relation_stop)?;
    }
    Ok(positions)
}

fn restrict(
    envelope: &mut CanonicalCommitEnvelope,
    full: &[crate::publication::patch::data::PublishedAuthoritativeRecordPatch],
    records: &[RecordRef],
    state: &WorkingState,
    registry: &crate::schema::data::RelationalSchemaRegistry,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<(), TransactionCommitError> {
    context
        .checkpoint(full.len().saturating_mul(records.len() + 1) as u64)
        .map_err(origin_request)?;
    let private_lookup = (usize::BITS
        - registry
            .aggregate_private_kinds
            .len()
            .max(1)
            .leading_zeros()) as u64;
    let mut selected = Vec::new();
    for patch in full {
        context
            .checkpoint(private_lookup + 1)
            .map_err(origin_request)?;
        // Native consequences are common to this actual candidate. Each route
        // captures only incident metadata through the existing relation owner.
        if records.contains(&patch.target)
            || crate::aggregate_identity::guard::is_metadata_record(registry, state, &patch.target)
        {
            selected.push(patch.clone());
        }
    }
    envelope.patch.authoritative_record_patches = selected;
    Ok(())
}

fn require_authored_sources(
    envelope: &CanonicalCommitEnvelope,
    custody: &crate::mvcc::RelationalAuthoredCoupledOwners,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<(), TransactionCommitError> {
    use crate::publication::{
        data::{AuthoredRetirementRefusal as Cause, PublicationError},
        patch::data::RecordStructuralChange as Change,
    };
    let anchors = std::iter::once(custody.anchor)
        .chain(custody.co_anchors.iter().map(|(id, _)| *id))
        .collect::<Vec<_>>();
    for id in &anchors {
        context
            .checkpoint(envelope.patch.authoritative_record_patches.len() as u64 + 1)
            .map_err(origin_request)?;
        if !envelope
            .patch
            .authoritative_record_patches
            .iter()
            .any(|p| p.target == RecordRef::Entity(*id) && p.structural_change == Change::Deleted)
        {
            return Err(TransactionCommitError::publication(
                PublicationError::authored_retirement(Cause::MissingMember, RecordRef::Entity(*id)),
            ));
        }
    }
    for patch in &envelope.patch.authoritative_record_patches {
        context
            .checkpoint(anchors.len() as u64 + 1)
            .map_err(origin_request)?;
        if let RecordRef::Entity(id) = patch.target
            && patch.structural_change == Change::Deleted
            && !anchors.contains(&id)
        {
            return Err(TransactionCommitError::publication(
                PublicationError::authored_retirement(Cause::ExtraEntity, RecordRef::Entity(id)),
            ));
        }
    }
    Ok(())
}
