use contracts::transactions::data::outcomes::commit_preparation::CommitPreparationErrorOwnerPort as _;
use worth_relational_branch::internal_port::history::data::canonical_commit_envelope::CanonicalCommitEnvelopeBodyFieldsPort as _;
use worth_relational_contracts::internal_port as contracts;
pub mod application_effect;
pub mod source_history;
use super::super::phase_backing::CommitPhaseBacking;
pub mod current_ownership;
pub mod owned_record_custody;
pub mod ownership_completion;
pub mod preparation_denial;
pub mod root_admission;
pub mod semantic_summary;
pub mod validated_lineage;
use super::PublicationPhaseOwners;
use crate::runtime::state::runtime_state::preparation_runtime::mirror_publication::prepare_mirror_publication;
use contracts::publication::bundle::publication_status::PublicationStage;
use contracts::publication::data::publication_error::PublicationError;
use contracts::transactions::data::outcomes::transaction_commit_error::{
    TransactionCommitError, TransactionCommitErrorOwnerPort as _,
};
use std::sync::Arc;
pub mod input;
pub(super) use input::PreparePublicationPhaseInput;
pub mod prepared_phase;
pub(super) use prepared_phase::{PreparedPublicationCompletion, PreparedPublicationPhase};
pub(super) fn prepare_publication_phase<
    Attach: FnOnce(
        &worth_relational_branch::internal_port::branch::root::PreparedRelationalBranchRootCapture,
    ),
>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    owners: PublicationPhaseOwners<'_, Attach>,
    input: PreparePublicationPhaseInput<'_>,
) -> Result<CommitPhaseBacking<PreparedPublicationPhase>, TransactionCommitError> {
    let result = {
        let PublicationPhaseOwners {
            root,
            work,
            diagnostics,
            lineage,
            fallback_schema,
            invariant_generation,
            max_patch_records,
            mirror,
            ..
        } = owners;
        let history_owner = root.history;
        let symbols = root.symbols;
        let storage = root.storage;
        let runtime_identity = root.runtime_identity;
        let PreparePublicationPhaseInput {
            lineage_deferred,
            naming_plan,
            structural,
            validation,
            caller_association,
            source_interpretation,
            recovered_codec_publication_custody,
            required_output_publication,
            required_output_materialization,
            created_entities,
            created_relations,
            commit_log,
            mut working_state,
            mut mutation_identities,
            selected_branch_state,
            publication,
            version_id,
            previous_branch_head_version,
            commit_id,
            commit_reference,
            branch_basis,
            branch_id,
        } = input;
        let (
            aggregate_partitions,
            mut artifacts,
            mut changed_records,
            mut canonical_commit_envelope,
            adjacency_deltas,
            mut lineage_nodes,
            mut relation_lineages,
            deferred_diagnostic_artifacts,
            target_schema_registry,
        ) = publication.into_inner().into_finalize().into_parts();
        let application_unchanged = application_effect::unchanged(
            execution,
            &changed_records,
            &adjacency_deltas,
            &canonical_commit_envelope,
            &mutation_identities.symbols,
        )?;
        let current_ownership = current_ownership::select(
            execution,
            work,
            &branch_basis,
            &canonical_commit_envelope,
            created_entities,
            &adjacency_deltas,
            &working_state,
            required_output_publication.as_ref(),
        )?;
        if required_output_materialization.is_some()
            && (recovered_codec_publication_custody.is_some()
                || required_output_publication.is_some())
        {
            return Err(
                contracts::transactions::data::outcomes::transaction_commit_error::origin_denial(
                    "materialization maintenance cannot also publish producer or codec retirement meaning",
                ),
            );
        }
        (*canonical_commit_envelope.owner_required_output_materialization_mut()) =
            required_output_materialization;
        if recovered_codec_publication_custody.is_some() && required_output_publication.is_some() {
            return Err(
                contracts::transactions::data::outcomes::transaction_commit_error::origin_denial(
                    "codec retirement cannot also mint a producer completion",
                ),
            );
        }
        let mut live_inputs =
            crate::publication::live_publication_inputs::LivePublicationInputs::default();
        let mut output_positions = contracts::execution::request_work::transaction_stage(
            execution,
            work,
            || {
                Ok::<_, TransactionCommitError>(
                    if let Some(custody) = recovered_codec_publication_custody {
                        Some(custody.install_retirements(
                            execution,
                            &mut canonical_commit_envelope,
                            &branch_basis,
                        )?)
                    } else if let Some(admission) = required_output_publication {
                        Some(crate::history::data::canonical_commit_envelope::required_output_origin::installation::complete::install_required_output_origin(
                            &mut canonical_commit_envelope,
                            execution,
                            crate::history::data::canonical_commit_envelope::required_output_origin::history_request::OutputPreparationInputs {
                                current_ownership: &current_ownership,
                                history: history_owner,
                                runtime_instance_id: runtime_identity,
                                custom_invariant_generation: invariant_generation,
                            },
                            &branch_basis,
                            created_entities,
                            created_relations,
                            &working_state,
                            admission,
                            &mut live_inputs,
                        )?)
                    } else {
                        None
                    },
                )
            },
        )?;
        let check_custody = || {
            owned_record_custody::require(
                execution,
                current_ownership.as_deref(),
                &canonical_commit_envelope,
                &adjacency_deltas,
            )
        };
        contracts::execution::request_work::transaction_stage(execution, work, check_custody)?;
        let (_current_ownership_funding, additional_aspects, additional_adjacency) =
            ownership_completion::complete(
                execution,
                ownership_completion::OwnershipCompletionInputs {
                    current: current_ownership.as_deref(),
                    storage,
                    symbols,
                    diagnostics,
                    work,
                    max_patch_records,
                },
                &branch_basis,
                &selected_branch_state,
                &mut working_state,
                &mut mutation_identities,
                &mut canonical_commit_envelope,
                &mut artifacts,
                &mut changed_records,
                &mut output_positions,
            )?;
        let lineage_identities = lineage.identities;
        if lineage_deferred {
            (lineage_nodes, relation_lineages) = prepared_phase::complete_deferred_lineage(
                execution,
                lineage,
                &mut working_state,
                &mut canonical_commit_envelope,
                &changed_records,
                created_entities,
                created_relations,
                naming_plan,
                &aggregate_partitions,
            )?;
        }
        crate::authority::commit::phases::publication::enforce_patch_budget(
            max_patch_records,
            diagnostics,
            &canonical_commit_envelope.patch,
        )?;
        semantic_summary::complete(
            commit_log,
            &canonical_commit_envelope,
            &changed_records,
            &additional_aspects,
            additional_adjacency,
        )?;
        (*canonical_commit_envelope.owner_caller_association_mut()) = caller_association;
        let semantic_result = super::super::semantic_result::prepare_semantic_result(
            symbols,
            super::super::semantic_result::SemanticResultInput {
                structural,
                validation,
                entities: created_entities,
                relations: created_relations,
                log: commit_log,
                changed: &changed_records,
                envelope: &canonical_commit_envelope,
            },
        )?;
        let canonical_commit_envelope =
            canonical_commit_envelope
                .seal(semantic_result)
                .map_err(|refusal| {
                    TransactionCommitError::preparation(
                        contracts::transactions::data::outcomes::commit_preparation::CommitPreparationError::semantic_result(
                            branch_id.clone(),
                            version_id,
                            refusal,
                        ),
                    )
                })?;
        let canonical_commit_envelope =
            contracts::execution::request_work::transaction_stage(execution, work, || {
                source_history::freeze(canonical_commit_envelope, source_interpretation, *execution)
            })?;
        let live_inputs = live_inputs.bind(&canonical_commit_envelope);
        let clone_mode = working_state.clone_mode();
        let committed_partitions = working_state.into_partition_commits().1;
        history_owner
            .validate_branch_root_capture(committed_partitions.len())
            .map_err(|denial| {
                TransactionCommitError::publication(PublicationError::new(
                    PublicationStage::BundleAssembly,
                    format!("branch-root capture preflight denied: {denial:?}"),
                ))
                .with_commit_log(commit_log.clone())
            })?;
        let published_partition_delta =
            worth_relational_storage::internal_port::storage::overlay::published_delta::RelationalPublishedPartitionDelta::from_committed_partitions(
                &committed_partitions,
            );
        let publication_schema_registry = target_schema_registry
            .or_else(|| {
                selected_branch_state
                    .root()
                    .map(|root| root.schema_authority().registry_arc())
            })
            .unwrap_or_else(|| {
                worth_relational_branch::internal_port::branch::root::schema::compiled_retention::RetainedRelationalSchemaRegistry::from_configuration(
                    std::sync::Arc::new(fallback_schema.clone()),
                )
            });
        let prepared_history = root_admission::run(execution, work, || {
            root.prepare_commit_publication(
                execution,
                commit_id,
                commit_reference.clone(),
                &branch_basis,
                &selected_branch_state,
                published_partition_delta,
                Arc::clone(&canonical_commit_envelope),
                publication_schema_registry.as_ref(),
                output_positions,
            )
        })
        .map_err(|denial| {
            preparation_denial::transaction_error(denial, &branch_basis, commit_log.clone())
        })?;
        lineage_identities
            .validate_reserved_lineage_events(canonical_commit_envelope.lineage_events())
            .map_err(|detail| {
                TransactionCommitError::publication(PublicationError::new(
                    PublicationStage::BundleAssembly,
                    detail,
                ))
                .with_commit_log(commit_log.clone())
            })?;
        let validated_lineage_events =
            validated_lineage::validated_lineage_events(&canonical_commit_envelope, commit_log)?;
        let reserve_route = |execution: worth_execution::ExecutionRequest<'_, '_>| {
            history_owner.reserve_canonical_publication_route(
                execution,
                prepared_history.envelope_residency(),
                Arc::clone(prepared_history.root()),
            )
        };
        let canonical_publication_route = match (
            contracts::execution::request_custody::request_lease(execution),
            work,
        ) {
            (Some(_), Some(budget)) => budget
                .run_stage(*execution, || reserve_route(*execution))
                .map_err(|stop| TransactionCommitError::execution(stop.into()))
                .and_then(std::convert::identity),
            _ => reserve_route(*execution),
        }
        .map_err(|error| error.with_commit_log(commit_log.clone()))?;

        let symbol_publication =
            worth_relational_branch::internal_port::branch::symbol_publication::SymbolPublicationAdmission::prepare(
                prepared_history.root(),
            )
            .map_err(|failure| {
                worth_relational_branch::internal_port::authority::commit::preparation::storage_denial::storage_read_denial(&branch_basis, failure)
            })?;
        let mirror_publication = contracts::execution::request_work::transaction_stage(
            execution,
            work,
            || {
                prepare_mirror_publication(
                mirror,
                symbols,
                execution,
                &branch_id,
                prepared_history.root(),
                &symbol_publication,
                clone_mode,
                committed_partitions,
            )
            .map_err(|failure| {
                worth_relational_branch::internal_port::authority::commit::preparation::storage_denial::storage_read_denial(&branch_basis, failure)
            })
            },
        )?;
        symbols.register_symbol_publication(symbol_publication.key());
        let symbol_meanings = mutation_identities
            .symbols
            .seal_for_publication(
                canonical_commit_envelope.owner_symbol_texts(),
                symbols,
                symbol_publication.required(),
            )
            .map_err(|detail| {
                TransactionCommitError::publication(PublicationError::new(
                    PublicationStage::BundleAssembly,
                    detail,
                ))
                .with_commit_log(commit_log.clone())
            })?;
        PreparedPublicationPhase {
            live_inputs: Some(live_inputs),
            application_unchanged,
            record_allocations: mutation_identities.records,
            symbol_meanings,
            _candidate_symbols: symbols.clone(),
            mirror_publication,
            prepared_history,
            validated_lineage_events,
            lineage_nodes,
            relation_lineages,
            artifacts,
            changed_records,
            version_id,
            previous_branch_head_version,
            commit_id,
            commit_reference,
            branch_id,
            deferred_diagnostic_artifacts,
            canonical_publication_route,
        }
    };
    CommitPhaseBacking::new(execution, result)
}
#[cfg(test)]
#[path = "preparation_inline_type_budget.rs"]
pub mod inline_type_budget;
