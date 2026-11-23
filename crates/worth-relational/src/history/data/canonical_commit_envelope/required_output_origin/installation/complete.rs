//! Complete one live required-output publication against the admitted branch.
use super::*;

#[allow(clippy::too_many_arguments)]
pub(crate) fn install_required_output_origin(
    envelope: &mut CanonicalCommitEnvelopeBody,
    execution: &ExecutionRequest<'_, '_>,
    runtime: &crate::runtime::RelationalPreparationRuntime,
    basis: &AdmittedRelationalBranchBasis,
    created: &CommitCreatedEntityBindings,
    created_relations: &CommitCreatedRelationBindings,
    state: &WorkingState,
    admission: crate::mvcc::RequiredOutputPublicationAdmission,
    live_inputs: &mut crate::publication::live_publication_inputs::LivePublicationInputs,
) -> Result<crate::history::data::RequiredOutputPublicationPositions, TransactionCommitError> {
    admission
        .installed
        .demand_source_scope(execution, basis, &admission.publication)?;
    let runtime = &crate::history::data::canonical_commit_envelope::required_output_origin::history_request::PreparationRuntimeRequest::new(runtime, execution);
    let result = execution
        .in_scope(|lease| {
            worth_execution::ExecutionWorkCeiling::new(execution.work_ceiling())
                .run_with_checkpoint(lease, None, |context| {
                    let mut origin_frames = OutputOriginHistory::default();
                    let validated_history = &mut origin_frames;
                    install_completion(
                        envelope,
                        execution,
                        runtime,
                        basis,
                        created,
                        created_relations,
                        state,
                        admission,
                        None,
                        None,
                        Some(live_inputs),
                        lease,
                        validated_history,
                        context,
                    )
                })
        })
        .map_err(origin_request)?
        .map_err(origin_request)?
        .0;
    execution.consult().map_err(origin_request)?;
    result
}

#[allow(clippy::too_many_arguments)]
pub(in crate::history::data::canonical_commit_envelope::required_output_origin) fn install_completion(
    envelope: &mut CanonicalCommitEnvelopeBody,
    execution: &ExecutionRequest<'_, '_>,
    runtime: &crate::history::data::canonical_commit_envelope::required_output_origin::history_request::PreparationRuntimeRequest<'_, '_, '_>,
    basis: &AdmittedRelationalBranchBasis,
    created: &CommitCreatedEntityBindings,
    created_relations: &CommitCreatedRelationBindings,
    state: &WorkingState,
    admission: crate::mvcc::RequiredOutputPublicationAdmission,
    contribution_records: Option<&[crate::transactions::data::primitives::RecordRef]>,
    source_input: Option<
        &crate::runtime::RelationalPinnedRead<
            crate::runtime::RelationalAppliedCandidateReadIdentity,
        >,
    >,
    live_inputs: Option<&mut crate::publication::live_publication_inputs::LivePublicationInputs>,
    lease: Option<&worth_execution::ExecutionResourceLease<'_>>,
    validated_history: &mut OutputOriginHistory,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<crate::history::data::RequiredOutputPublicationPositions, TransactionCommitError> {
    let crate::mvcc::RequiredOutputPublicationAdmission {
        inputs,
        installed,
        publication,
    } = admission;
    context
        .checkpoint(publication.admission_work())
        .map_err(origin_request)?;
    if installed.runtime_instance_id != runtime.runtime.runtime_instance_id()
        || installed.custom_invariant_generation
            != runtime
                .runtime
                .schema_contract_runtime
                .custom_invariant_generation
        || installed.schema_digest != basis.inner.root.schema_authority().authority_digest()
        || !publication.admitted_by(&installed)
        || envelope.branch_context != *basis.descriptor().branch_id()
        || !envelope.is_unmerged_output_publication_step()
        || envelope.schema_transition.is_some()
        || envelope.schema_continuation_descriptor.is_some()
        || envelope.schema_reconciliation_descriptor.is_some()
    {
        return Err(
            crate::transactions::data::outcomes::transaction_commit_error::origin_denial(
                "required-output installation changed before native preparation",
            ),
        );
    }
    if let Publication::HistoricalSelection {
        admission,
        selections,
    } = publication
    {
        return super::super::historical_selection::install(
            envelope, basis, state, installed, &admission, selections, lease, context,
        );
    }
    if let Publication::CoupledAuthored { group } = publication {
        return super::super::coupled_authored_install::install(
            envelope,
            execution,
            runtime,
            basis,
            created,
            created_relations,
            state,
            installed,
            group,
            live_inputs.expect("only top-level installation admits a coupled publication"),
            lease,
            validated_history,
            context,
        );
    }
    if let Publication::AuthoredCascadeRestoration {
        admission,
        restoration,
    } = publication
    {
        return super::super::authored_restoration::install(
            envelope,
            runtime,
            basis,
            created,
            created_relations,
            state,
            installed,
            Some(&admission),
            &restoration,
            lease,
            validated_history,
            context,
        );
    }
    if let Publication::AuthoredCascadeRetirement {
        admission,
        predecessor,
    } = publication
    {
        return super::super::authored_cascade_install::install(
            envelope,
            execution,
            runtime,
            basis,
            created,
            created_relations,
            state,
            installed,
            &admission,
            predecessor,
            lease,
            validated_history,
            context,
        );
    }
    let (producer, dependency_identity) = publication.lineage_attribution().ok_or_else(|| {
        crate::transactions::data::outcomes::transaction_commit_error::origin_denial(
            "cascade attribution requires per-publication provenance",
        )
    })?;
    let mut after_codec_retirement = None;
    let (meaning, roles, previous) = match &publication {
        Publication::Birth {
            meaning,
            roles,
            absence,
            ..
        } => {
            after_codec_retirement =
                crate::mvcc::RequiredOutputPublicationAdmission::birth_retirement_in_context(
                    &installed,
                    basis,
                    meaning,
                    absence.as_ref(),
                    context,
                )?;
            (meaning, roles, None)
        }
        Publication::Preserve {
            predecessor,
            meaning,
            roles,
            ..
        } => {
            let origin = predecessor::validate(
                envelope,
                runtime,
                basis,
                predecessor,
                meaning,
                lease,
                validated_history,
                context,
            )?;
            (meaning, roles, Some(origin))
        }
        Publication::HistoricalSelection { .. }
        | Publication::CoupledAuthored { .. }
        | Publication::AuthoredCascadeRetirement { .. }
        | Publication::AuthoredCascadeRestoration { .. } => {
            unreachable!("cascade consumed before ordinary installation")
        }
        Publication::AuthoredRetirement {
            predecessor, roles, ..
        } => {
            let meaning = &predecessor.origin.meaning;
            let origin = predecessor::validate(
                envelope,
                runtime,
                basis,
                predecessor,
                meaning,
                lease,
                validated_history,
                context,
            )?;
            super::super::source_scope_ownership::require_cascade(
                origin,
                envelope.authored_cascade_retirement.as_deref(),
            )?;
            predecessor::require_complete_retirement(origin, roles)?;
            (meaning, roles, Some(origin))
        }
    };
    if let Publication::Preserve {
        admission: succession,
        predecessor: generator,
        ..
    } = &publication
        && let Some(prior) = previous
        && let Some(closure) = producer_cascade::prepare(
            envelope,
            execution,
            basis,
            runtime,
            &installed,
            prior,
            roles,
            succession,
            generator.authority(),
            context,
        )?
    {
        return envelope.install_producer_cascade(
            runtime,
            basis,
            created,
            created_relations,
            state,
            &installed,
            route_completion::RouteCompletion {
                inputs,
                contribution_records,
                source_input,
                producer,
                dependency_identity,
                meaning,
                roles,
                previous,
                completion: CanonicalOutputPublication::Preserve {
                    predecessor: prior.commit.commit_id,
                },
            },
            closure,
            lease,
            validated_history,
            context,
        );
    }
    if let Some(prior) = previous
        && !matches!(publication, Publication::Preserve { .. })
    {
        scope_dependents::require_retained_sources(
            envelope, runtime, &installed, prior, roles, context,
        )?;
    }
    let completion = previous.map_or_else(
        || {
            after_codec_retirement.map_or(CanonicalOutputPublication::Birth, |retirement| {
                CanonicalOutputPublication::BirthAfterCodecRetirement { retirement }
            })
        },
        |prior| {
            if matches!(publication, Publication::AuthoredRetirement { .. }) {
                CanonicalOutputPublication::AuthoredRetirement {
                    predecessor: prior.commit.commit_id,
                }
            } else {
                CanonicalOutputPublication::Preserve {
                    predecessor: prior.commit.commit_id,
                }
            }
        },
    );
    envelope.install_route(
        runtime,
        basis,
        created,
        created_relations,
        state,
        &installed,
        route_completion::RouteCompletion {
            inputs,
            contribution_records,
            source_input,
            producer,
            dependency_identity,
            meaning,
            roles,
            previous,
            completion,
        },
        lease,
        validated_history,
        context,
    )
}
