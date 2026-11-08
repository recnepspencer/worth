//! Canonical coupled commits retain each genuine producer route and provenance.
use super::*;
use crate::{
    history::data::{RequiredOutputPublicationPositions, RequiredOutputRoutePosition},
    mvcc::RelationalCoupledAuthoredPublication,
    storage::overlay::WorkingState,
    transactions::data::{CommitCreatedEntityBindings, CommitCreatedRelationBindings, RecordRef},
};
use std::sync::Arc;
use worth_execution::{ExecutionResourceLease, MapKernelContext};

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum CanonicalCoupledAuthoredSources {
    Retirement(Arc<CanonicalAuthoredRetirement>),
    Restoration(Arc<CanonicalAuthoredRestoration>),
    Edit(crate::identity::data::EntityId),
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
    #[serde(skip)]
    pub(crate) live_inputs: crate::history::data::LivePublicationInputs,
}
impl CanonicalCoupledAuthoredPublication {
    pub(crate) fn output_origins(
        &self,
    ) -> impl Iterator<Item = &Arc<CanonicalRequiredOutputOrigin>> {
        let sources: &[Arc<CanonicalRequiredOutputOrigin>] = match &self.sources {
            CanonicalCoupledAuthoredSources::Retirement(event) => &event.publications,
            CanonicalCoupledAuthoredSources::Restoration(event) => &event.publications,
            CanonicalCoupledAuthoredSources::Edit(_) => &[],
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
                CanonicalCoupledAuthoredSources::Restoration(_)
                | CanonicalCoupledAuthoredSources::Edit(_) => None,
            }
        })
    }
    pub(crate) fn authored_source_restoration(&self) -> Option<&Arc<CanonicalAuthoredRestoration>> {
        self.authored_cascade_restoration.as_ref().or_else(|| {
            match &self.coupled_authored_publication.as_ref()?.sources {
                CanonicalCoupledAuthoredSources::Restoration(event) => Some(event),
                CanonicalCoupledAuthoredSources::Retirement(_)
                | CanonicalCoupledAuthoredSources::Edit(_) => None,
            }
        })
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn install(
    envelope: &mut CanonicalCommitEnvelope,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &crate::runtime::RelationalPreparationRuntime,
    basis: &AdmittedRelationalBranchBasis,
    created: &CommitCreatedEntityBindings,
    created_relations: &CommitCreatedRelationBindings,
    state: &WorkingState,
    installed: crate::mvcc::RelationalInstalledApplicationPublicationAuthority,
    group: RelationalCoupledAuthoredPublication,
    lease: Option<&ExecutionResourceLease<'_>>,
    validated_history: &mut OutputOriginHistory,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<RequiredOutputPublicationPositions, TransactionCommitError> {
    if group.parts.is_empty() {
        // All declared roots were already members of the admitted retirement
        // closure. Use its existing canonical event, with unchanged bytes.
        return super::coupled_source::install_empty(
            envelope,
            execution,
            runtime,
            basis,
            created,
            created_relations,
            state,
            installed,
            &group,
            lease,
            validated_history,
            context,
        );
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
        let sources = super::coupled_source::install(
            envelope,
            execution,
            runtime,
            basis,
            created,
            created_relations,
            state,
            installed.clone(),
            &group,
            lease,
            validated_history,
            context,
        )?;
        let mut participants = Vec::with_capacity(group.parts.len());
        let mut live_inputs = crate::history::data::LivePublicationInputs::default();
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
                execution,
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
                Some(&part.input),
                lease,
                validated_history,
                context,
            )?;
            let origin = envelope
                .required_output_origin
                .take()
                .expect("Native installed coupled successor");
            live_inputs.insert(Arc::clone(&origin), part.input);
            participants.push(CanonicalCoupledParticipant {
                origin,
                retirement: envelope.producer_cascade_retirement.take(),
            });
        }
        let event = CanonicalCoupledAuthoredPublication {
            sources,
            participants,
            live_inputs,
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
    let current = envelope
        .validate_output_origins(context)
        .map_err(super::installation::relation_stop)?;
    for origin in envelope.output_origins() {
        crate::history::data::publication_relations::validate(
            &basis.inner.root,
            Some(state),
            origin,
            Some(
                crate::history::data::publication_relations::CurrentPublicationFrame::Complete(
                    current,
                ),
            ),
            &mut |commit| runtime.history.canonical_envelope(commit),
            &mut |version| runtime.history.canonical_envelope_for_version(version),
            runtime.history.canonical_catalog_len(),
            lease,
            validated_history,
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
    let private_lookup =
        (usize::BITS - registry.native_private_kinds.len().max(1).leading_zeros()) as u64;
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
