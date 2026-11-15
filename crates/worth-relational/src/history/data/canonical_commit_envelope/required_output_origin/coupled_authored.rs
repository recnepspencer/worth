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
    /// The prefix source followed by every consumed later stage, in apply order.
    Edit {
        sources: Vec<crate::identity::data::EntityId>,
    },
    Retirement {
        prefix: Arc<CanonicalAuthoredRetirement>,
        later: Vec<crate::identity::data::EntityId>,
    },
    Restoration {
        prefix: Arc<CanonicalAuthoredRestoration>,
        later: Vec<crate::identity::data::EntityId>,
    },
    MemberEditRetirement {
        sources: Vec<crate::identity::data::EntityId>,
        prefix: Arc<CanonicalAuthoredRetirement>,
    },
    MemberEditRestoration {
        sources: Vec<crate::identity::data::EntityId>,
        prefix: Arc<CanonicalAuthoredRestoration>,
    },
}
impl CanonicalCoupledAuthoredSources {
    pub(crate) fn edit_sources(&self) -> Option<&[crate::identity::data::EntityId]> {
        match self {
            Self::Edit { sources }
            | Self::MemberEditRetirement { sources, .. }
            | Self::MemberEditRestoration { sources, .. } => Some(sources),
            _ => None,
        }
    }
    pub(crate) fn is_edit(&self) -> bool {
        self.edit_sources().is_some()
    }
}
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum CanonicalCoupledParticipation {
    Edit,
    Required,
}
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct CanonicalCoupledParticipant {
    pub(crate) origin: Arc<CanonicalRequiredOutputOrigin>,
    pub(crate) retirement: Option<Arc<CanonicalProducerRetirement>>,
    pub(crate) participation: CanonicalCoupledParticipation,
}
impl CanonicalCoupledParticipant {
    pub(crate) fn is_required(&self) -> bool {
        matches!(self.participation, CanonicalCoupledParticipation::Required)
    }
}
#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct CanonicalCoupledAuthoredPublication {
    pub(crate) sources: CanonicalCoupledAuthoredSources,
    pub(crate) participants: Vec<CanonicalCoupledParticipant>,
    #[serde(skip)]
    pub(crate) live_inputs: crate::history::data::LivePublicationInputs,
    #[serde(skip)]
    pub(crate) _decoded_memory: Option<Arc<worth_execution::ExecutionMemoryReservation>>,
}
impl CanonicalCoupledAuthoredPublication {
    pub(crate) fn participant_decode_storage(capacity: usize) -> Option<u64> {
        u64::try_from(
            std::alloc::Layout::array::<CanonicalCoupledParticipant>(capacity)
                .ok()?
                .size(),
        )
        .ok()
    }
    pub(crate) fn output_origins(
        &self,
    ) -> impl Iterator<Item = &Arc<CanonicalRequiredOutputOrigin>> {
        let sources: &[Arc<CanonicalRequiredOutputOrigin>] = match &self.sources {
            CanonicalCoupledAuthoredSources::Retirement { prefix, .. }
            | CanonicalCoupledAuthoredSources::MemberEditRetirement { prefix, .. } => {
                &prefix.publications
            }
            CanonicalCoupledAuthoredSources::Restoration { prefix, .. }
            | CanonicalCoupledAuthoredSources::MemberEditRestoration { prefix, .. } => {
                &prefix.publications
            }
            CanonicalCoupledAuthoredSources::Edit { .. } => &[],
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

impl CanonicalCommitEnvelopeBody {
    pub(crate) fn authored_source_retirement(&self) -> Option<&Arc<CanonicalAuthoredRetirement>> {
        self.authored_cascade_retirement.as_ref().or_else(|| {
            match &self.coupled_authored_publication.as_ref()?.sources {
                CanonicalCoupledAuthoredSources::Retirement { prefix, .. }
                | CanonicalCoupledAuthoredSources::MemberEditRetirement { prefix, .. } => {
                    Some(prefix)
                }
                _ => None,
            }
        })
    }
    pub(crate) fn authored_source_restoration(&self) -> Option<&Arc<CanonicalAuthoredRestoration>> {
        self.authored_cascade_restoration.as_ref().or_else(|| {
            match &self.coupled_authored_publication.as_ref()?.sources {
                CanonicalCoupledAuthoredSources::Restoration { prefix, .. }
                | CanonicalCoupledAuthoredSources::MemberEditRestoration { prefix, .. } => {
                    Some(prefix)
                }
                _ => None,
            }
        })
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn install(
    envelope: &mut CanonicalCommitEnvelopeBody,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &crate::history::data::canonical_commit_envelope::required_output_origin::history_request::PreparationRuntimeRequest<'_, '_, '_>,
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
    crate::execution::request_stage::checkpoint(context, group.parts.len() as u64)
        .map_err(origin_request)?;
    if group
        .parts
        .iter()
        .all(|part| part.publication.owned().is_none())
    {
        // With no owned participants, retain the existing authored source event.
        // An ordinary Edit emits no coupled event or publication origin.
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
        // Retain the one necessary copy of the actually consumed source roster.
        let source_count = group
            .stages
            .len()
            .saturating_add(usize::from(group.binding.owners.source.is_edit()));
        let source_memory = crate::mvcc::plan_copy_admission::reserve_copy(
            execution,
            crate::mvcc::plan_copy_admission::slots::<crate::identity::data::EntityId>(
                source_count,
            ),
        )?
        .map(Arc::new);
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
            let crate::mvcc::RelationalCoupledProducerPosture::Publication(publication) =
                part.publication
            else {
                continue;
            };
            let meaning = publication
                .producing_meaning()
                .expect("owned participant has a route");
            crate::execution::request_stage::checkpoint(
                context,
                group
                    .binding
                    .owners
                    .required_owners()
                    .iter()
                    .map(|owner| {
                        (owner.origin.meaning.family.len() as u64)
                            .saturating_add(meaning.family.len() as u64)
                            .saturating_add(65)
                    })
                    .fold(0u64, u64::saturating_add),
            )
            .map_err(origin_request)?;
            let participation = if group
                .binding
                .owners
                .required_owners()
                .iter()
                .any(|owner| owner.origin.meaning.same_route(meaning))
            {
                CanonicalCoupledParticipation::Required
            } else {
                CanonicalCoupledParticipation::Edit
            };
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
                    publication,
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
                participation,
            });
        }
        let event = CanonicalCoupledAuthoredPublication {
            _decoded_memory: source_memory,
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
            &mut |version, context| {
                runtime
                    .history
                    .canonical_envelope_for_version(version, context)
            },
            runtime.history.canonical_catalog_len(),
            lease,
            validated_history,
            &runtime.history.execution(),
            context,
        )
        .map_err(super::installation::relation_stop)?;
    }
    Ok(positions)
}

fn restrict(
    envelope: &mut CanonicalCommitEnvelopeBody,
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

// Live input pins and decode backing are transient custody, not publication meaning.
impl PartialEq for CanonicalCoupledAuthoredPublication {
    fn eq(&self, other: &Self) -> bool {
        self.sources == other.sources && self.participants == other.participants
    }
}
impl Eq for CanonicalCoupledAuthoredPublication {}
