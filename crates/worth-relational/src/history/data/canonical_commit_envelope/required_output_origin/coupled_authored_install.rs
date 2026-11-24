//! Complete the group and stage the applied pins for the performed publication routes.
use worth_relational_branch::internal_port::history::data::canonical_commit_envelope::required_output_origin::group_selection::CanonicalCommitEnvelopeBodyOwnerPort as _;
use worth_relational_branch::internal_port::history::data::canonical_commit_envelope::CanonicalCommitEnvelopeBodyFieldsPort as _;
use worth_relational_branch::internal_port::branch::basis::AdmittedRelationalBranchBasisFieldsPort as _;
use worth_relational_contracts::internal_port::schema::data::RelationalSchemaRegistryFieldsPort as _;
use worth_relational_contracts::internal_port::mvcc::publication::application_publication::output_meaning::RequiredOutputPublicationMeaningOwnerPort as _;
use super::*;
use crate::branch::basis::AdmittedRelationalBranchBasis;
use crate::history::data::RequiredOutputPublicationPositions;
use crate::history::data::RequiredOutputRoutePosition;
use crate::mvcc::publication::application_publication::RelationalCoupledAuthoredPublication;
use crate::storage::overlay::WorkingState;
use crate::transactions::data::outcomes::created_entity_bindings::CommitCreatedEntityBindings;
use crate::transactions::data::outcomes::created_relation_bindings::CommitCreatedRelationBindings;
use crate::transactions::data::primitives::RecordRef;
use std::sync::Arc;
use worth_execution::{ExecutionResourceLease, MapKernelContext};

use super::coupled_authored::{
    CanonicalCoupledAuthoredPublication, CanonicalCoupledParticipant, CanonicalCoupledParticipation,
};

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
        let source_memory = crate::mvcc::transaction::normalized_plan::reserve_copy(
            execution,
            crate::mvcc::transaction::normalized_plan::slots::<crate::identity::data::EntityId>(
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
        .map_err(super::installation::relation_stop)?;
    for origin in envelope.output_origins() {
        crate::history::data::publication_relations::validate(
            &basis.owner_inner().root,
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
            || crate::storage::overlay::record_observation::is_metadata_record(
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
