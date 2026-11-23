mod application_effect;
mod source_history;
use super::super::phase_backing::CommitPhaseBacking;
mod current_ownership;
mod owned_record_custody;
mod ownership_completion;
mod preparation_denial;
mod root_admission;
mod semantic_summary;
use std::sync::Arc;

use crate::publication::bundle::PublicationStage;
use crate::publication::data::PublicationError;
use crate::runtime::RelationalPreparationRuntime;
use crate::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;

mod input;
pub(super) use input::PreparePublicationPhaseInput;
mod prepared_phase;
pub(super) use prepared_phase::{PreparedPublicationCompletion, PreparedPublicationPhase};

pub(super) fn prepare_publication_phase(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalPreparationRuntime,
    input: PreparePublicationPhaseInput<'_>,
) -> Result<CommitPhaseBacking<PreparedPublicationPhase>, TransactionCommitError> {
    let result = {
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
            merge_base_commits,
            merge_parent_branches,
        } = input;
        let (
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
            &merge_base_commits,
            &merge_parent_branches,
        )?;
        let current_ownership = current_ownership::select(
            execution,
            runtime.commit_work_budget.as_ref(),
            &branch_basis,
            &canonical_commit_envelope,
            created_entities,
            &adjacency_deltas,
            &working_state,
            required_output_publication.as_ref(),
        )?;
        let runtime_with_current = runtime.with_current_ownership(current_ownership);
        let runtime = &runtime_with_current;
        if required_output_materialization.is_some()
            && (recovered_codec_publication_custody.is_some()
                || required_output_publication.is_some())
        {
            return Err(crate::transactions::data::outcomes::transaction_commit_error::origin_denial(
                "materialization maintenance cannot also publish producer or codec retirement meaning",
            ));
        }
        canonical_commit_envelope.required_output_materialization = required_output_materialization;
        if recovered_codec_publication_custody.is_some() && required_output_publication.is_some() {
            return Err(
                crate::transactions::data::outcomes::transaction_commit_error::origin_denial(
                    "codec retirement cannot also mint a producer completion",
                ),
            );
        }
        let mut live_inputs =
            crate::publication::live_publication_inputs::LivePublicationInputs::default();
        let mut output_positions = crate::execution::transaction_stage(
            execution,
            runtime.commit_work_budget.as_ref(),
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
                            runtime,
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
                runtime.current_ownership.as_deref(),
                &canonical_commit_envelope,
                &adjacency_deltas,
            )
        };
        crate::execution::transaction_stage(
            execution,
            runtime.commit_work_budget.as_ref(),
            check_custody,
        )?;
        let (_current_ownership_funding, additional_aspects, additional_adjacency) =
            ownership_completion::complete(
                execution,
                ownership_completion::OwnershipCompletionInputs {
                    current: runtime.current_ownership.as_deref(),
                    storage: &runtime.config.storage,
                    symbols: &runtime.services.symbols,
                    diagnostics: runtime.preparation_diagnostics(),
                    work: runtime.commit_work_budget.as_ref(),
                    max_patch_records: runtime
                        .config
                        .publication
                        .policy
                        .max_patch_records_per_commit,
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
        if lineage_deferred {
            (lineage_nodes, relation_lineages) = prepared_phase::complete_deferred_lineage(
                execution,
                runtime.lineage_preparation_authority(),
                &mut working_state,
                &mut canonical_commit_envelope,
                &changed_records,
                created_entities,
                created_relations,
                naming_plan,
            )?;
        }
        crate::authority::commit::phases::publication::enforce_patch_budget(
            runtime
                .config
                .publication
                .policy
                .max_patch_records_per_commit,
            runtime.preparation_diagnostics(),
            &canonical_commit_envelope.patch,
        )?;
        semantic_summary::complete(
            commit_log,
            &canonical_commit_envelope,
            &changed_records,
            &additional_aspects,
            additional_adjacency,
        )?;
        canonical_commit_envelope.caller_association = caller_association;
        let semantic_result = super::super::semantic_result::prepare_semantic_result(
            &runtime.services.symbols,
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
                        crate::transactions::data::outcomes::commit_preparation::CommitPreparationError::semantic_result(
                            branch_id.clone(),
                            version_id,
                            refusal,
                        ),
                    )
                })?;
        let canonical_commit_envelope = crate::execution::transaction_stage(
            execution,
            runtime.commit_work_budget.as_ref(),
            || source_history::freeze(canonical_commit_envelope, source_interpretation, *execution),
        )?;
        let live_inputs = live_inputs.bind(&canonical_commit_envelope);
        let clone_mode = working_state.clone_mode();
        let committed_partitions = working_state.into_partition_commits().1;
        runtime
            .history
            .validate_branch_root_capture(committed_partitions.len())
            .map_err(|denial| {
                TransactionCommitError::publication(PublicationError::new(
                    PublicationStage::BundleAssembly,
                    format!("branch-root capture preflight denied: {denial:?}"),
                ))
                .with_commit_log(commit_log.clone())
            })?;
        let published_partition_delta =
            crate::storage::RelationalPublishedPartitionDelta::from_committed_partitions(
                &committed_partitions,
            );
        let publication_schema_registry = target_schema_registry
            .or_else(|| {
                selected_branch_state
                    .root()
                    .map(|root| root.schema_authority().registry_arc())
            })
            .unwrap_or_else(|| {
                crate::branch::root::schema::compiled_retention::RetainedRelationalSchemaRegistry::from_configuration(
                    std::sync::Arc::new(runtime.config.schema.registry.clone()),
                )
            });
        let prepared_history =
            root_admission::run(execution, runtime.commit_work_budget.as_ref(), || {
                runtime.prepare_commit_publication(
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
        crate::branch::model_ownership::validate_candidate_ownership(
            execution,
            runtime,
            prepared_history.root(),
            &canonical_commit_envelope,
        )?;
        crate::publication::ownership::selection_pairing::validate(
            execution,
            prepared_history.root(),
            &canonical_commit_envelope,
        )?;
        runtime
            .validate_reserved_lineage_events(canonical_commit_envelope.lineage_events())
            .map_err(|detail| {
                TransactionCommitError::publication(PublicationError::new(
                    PublicationStage::BundleAssembly,
                    detail,
                ))
                .with_commit_log(commit_log.clone())
            })?;
        let validated_lineage_events =
            crate::runtime::state::subsystems::lineage::ValidatedLineageEventBatch::from_reserved(
                canonical_commit_envelope.lineage_events().to_vec(),
            );
        let reserve_route = |execution: worth_execution::ExecutionRequest<'_, '_>| {
            runtime.history.reserve_canonical_publication_route(
                execution,
                prepared_history.envelope_residency(),
                Arc::clone(prepared_history.root()),
            )
        };
        let canonical_publication_route = match (
            super::super::request_lease(execution),
            runtime.commit_work_budget.as_ref(),
        ) {
            (Some(_), Some(budget)) => budget
                .run_stage(*execution, || reserve_route(*execution))
                .map_err(|stop| TransactionCommitError::execution(stop.into()))
                .and_then(std::convert::identity),
            _ => reserve_route(*execution),
        }
        .map_err(|error| error.with_commit_log(commit_log.clone()))?;

        let symbol_publication =
            crate::branch::symbol_publication::SymbolPublicationAdmission::prepare(
                prepared_history.root(),
            )
            .map_err(|failure| {
                crate::authority::commit::preparation::storage_read_denial(&branch_basis, failure)
            })?;
        let mirror_publication = crate::execution::transaction_stage(
            execution,
            runtime.commit_work_budget.as_ref(),
            || {
                runtime
                    .prepare_mirror_publication(
                        execution,
                        &branch_id,
                        prepared_history.root(),
                        &symbol_publication,
                        clone_mode,
                        committed_partitions,
                    )
                    .map_err(|failure| {
                        crate::authority::commit::preparation::storage_read_denial(
                            &branch_basis,
                            failure,
                        )
                    })
            },
        )?;
        runtime
            .services
            .symbols
            .register_symbol_publication(symbol_publication.key());
        let symbol_meanings = mutation_identities
            .symbols
            .seal_for_publication(
                &canonical_commit_envelope.symbol_texts,
                &runtime.services.symbols,
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
            _candidate_symbols: runtime.services.symbols.clone(),
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
            merge_base_commits,
            merge_parent_branches,
            deferred_diagnostic_artifacts,
            canonical_publication_route,
        }
    };
    CommitPhaseBacking::new(execution, result)
}
#[cfg(test)]
#[path = "preparation_inline_type_budget.rs"]
pub(crate) mod inline_type_budget;
