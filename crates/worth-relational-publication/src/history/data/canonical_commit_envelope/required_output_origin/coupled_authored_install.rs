//! Complete the group and stage the applied pins for the performed publication routes.
use worth_relational_contracts::internal_port as contracts;
use worth_relational_branch::internal_port::history::data::canonical_commit_envelope::required_output_origin::group_selection::CanonicalCommitEnvelopeBodyOwnerPort as _;
use worth_relational_branch::internal_port::history::data::canonical_commit_envelope::CanonicalCommitEnvelopeBodyFieldsPort as _;
use worth_relational_branch::internal_port::branch::basis::{AdmittedRelationalBranchBasis, AdmittedRelationalBranchBasisFieldsPort as _};
use contracts::schema::data::RelationalSchemaRegistryFieldsPort as _;
use contracts::mvcc::publication::application_publication::output_meaning::RequiredOutputPublicationMeaningOwnerPort as _;
use super::*;
use worth_relational_branch::internal_port::history::data::required_output_positions::{RequiredOutputPublicationPositions, RequiredOutputRoutePosition};
use crate::mvcc::publication::application_publication::RelationalCoupledAuthoredPublication;
use worth_relational_storage::internal_port::storage::overlay::working_state::WorkingState;
use contracts::transactions::data::outcomes::created_entity_bindings::CommitCreatedEntityBindings;
use contracts::transactions::data::outcomes::created_relation_bindings::CommitCreatedRelationBindings;
use contracts::transactions::data::primitives::RecordRef;
use std::sync::Arc;
use worth_execution::{ExecutionResourceLease, MapKernelContext};

use worth_relational_branch::internal_port::history::data::canonical_commit_envelope::required_output_origin::coupled_authored::{CanonicalCoupledAuthoredPublication, CanonicalCoupledParticipant, CanonicalCoupledParticipation};

#[allow(clippy::too_many_arguments)]
pub(super) fn install(
    envelope: &mut CanonicalCommitEnvelopeBody,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &crate::history::data::canonical_commit_envelope::required_output_origin::history_request::OutputPreparationRequest<'_, '_, '_>,
    basis: &AdmittedRelationalBranchBasis,
    created: &CommitCreatedEntityBindings,
    created_relations: &CommitCreatedRelationBindings,
    state: &WorkingState,
    installed: crate::mvcc::publication::application_publication::RelationalInstalledApplicationPublicationAuthority,
    group: RelationalCoupledAuthoredPublication,
    live_inputs: &mut crate::publication::live_publication_inputs::LivePublicationInputs,
    lease: Option<&ExecutionResourceLease<'_>>,
    validated_history: &mut OutputOriginHistory,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<RequiredOutputPublicationPositions, TransactionCommitError> {
    contracts::execution::request_stage::checkpoint(context, group.parts.len() as u64)
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
        let source_memory = worth_relational_mutation::internal_port::mvcc::transaction::normalized_plan::reserve_copy(
            execution,
            worth_relational_mutation::internal_port::mvcc::transaction::normalized_plan::slots::<contracts::identity::data::record_ids::EntityId>(
                source_count,
            ),
        )?
        .map(Arc::new);
        restrict(
            envelope,
            &full,
            &group.authored_records,
            state,
            basis.owner_inner().root.schema_authority().registry(),
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
        for part in group.parts {
            let crate::mvcc::publication::application_publication::RelationalCoupledProducerPosture::Publication(publication) =
                part.publication
            else {
                continue;
            };
            let meaning = publication
                .producing_meaning()
                .expect("owned participant has a route");
            // A republished later-stage survivor keeps the Required recording
            // every non-edit participant has; only its presence is optional.
            let owners = &group.binding.owners;
            let recorded = || owners.required_owners().iter().chain(&owners.survivors);
            contracts::execution::request_stage::checkpoint(
                context,
                recorded()
                    .map(|owner| {
                        (owner.origin.meaning.family.len() as u64)
                            .saturating_add(meaning.family.len() as u64)
                            .saturating_add(65)
                    })
                    .fold(0u64, u64::saturating_add),
            )
            .map_err(origin_request)?;
            let participation = if recorded().any(|owner| owner.origin.meaning.same_route(meaning))
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
                basis.owner_inner().root.schema_authority().registry(),
                context,
            )?;
            super::installation::complete::install_completion(
                envelope,
                execution,
                runtime,
                basis,
                created,
                created_relations,
                state,
                crate::mvcc::publication::application_publication::RequiredOutputPublicationAdmission {
                    inputs: part.inputs.unwrap_or_default(),
                    installed: installed.clone(),
                    publication,
                },
                Some(&part.records),
                Some(&part.input),
                None,
                lease,
                validated_history,
                context,
            )?;
            let origin = (*envelope.owner_required_output_origin_mut())
                .take()
                .expect("Native installed coupled successor");
            live_inputs.insert(Arc::clone(&origin), part.input);
            participants.push(CanonicalCoupledParticipant {
                origin,
                retirement: (*envelope.owner_producer_cascade_retirement_mut()).take(),
                participation,
            });
        }
        let event = CanonicalCoupledAuthoredPublication {
            _decoded_memory: source_memory,
            sources,
            participants,
        };
        let mut positions = basis
            .owner_inner()
            .root
            .output_publication_positions
            .clone();
        for origin in event.output_origins() {
            positions.insert(
                &origin.meaning,
                RequiredOutputRoutePosition::Published(envelope.commit.commit_id),
                lease,
                context,
            )?;
        }
        (*envelope.owner_coupled_authored_publication_mut()) = Some(Arc::new(event));
        Ok(positions)
    })();
    envelope.patch.authoritative_record_patches = full;
    let positions = answer?;
    // Incidence sees the complete canonical birth frame, after every writer's
    // own roster is installed. A partial writer view cannot classify a later
    // writer's edge to an earlier output.
    let current = envelope
        .validate_output_origins(context)
        .map_err(worth_relational_branch::internal_port::history::data::canonical_commit_envelope::required_output_origin::installation::relation_failure::relation_stop)?;
    for origin in envelope.output_origins() {
        worth_relational_branch::internal_port::history::data::publication_relations::validate(
            &basis.root_read(),
            Some(state),
            origin,
            Some(
                worth_relational_branch::internal_port::history::data::publication_relations::current_frame::CurrentPublicationFrame::Complete(
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
        .map_err(worth_relational_branch::internal_port::history::data::canonical_commit_envelope::required_output_origin::installation::relation_failure::relation_stop)?;
    }
    Ok(positions)
}

fn restrict(
    envelope: &mut CanonicalCommitEnvelopeBody,
    full: &[contracts::publication::patch::data::authoritative_patch_envelope::PublishedAuthoritativeRecordPatch],
    records: &[RecordRef],
    state: &WorkingState,
    registry: &contracts::schema::data::RelationalSchemaRegistry,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<(), TransactionCommitError> {
    context
        .checkpoint(full.len().saturating_mul(records.len() + 1) as u64)
        .map_err(origin_request)?;
    let private_lookup = (usize::BITS
        - (*registry.owner_native_private_kinds())
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
            || worth_relational_storage::internal_port::storage::overlay::record_observation::is_metadata_record(
                registry,
                state,
                &patch.target,
            )
        {
            selected.push(patch.clone());
        }
    }
    envelope.patch.authoritative_record_patches = selected;
    Ok(())
}
