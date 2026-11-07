//! Pre-effect native membership installation and exact predecessor write fences.
use super::*;
use crate::mvcc::RequiredOutputPublication as Publication;
use crate::storage::overlay::WorkingState;
use crate::transactions::data::{CommitCreatedEntityBindings, CommitCreatedRelationBindings};
use worth_execution::{ExecutionRequest, MapKernelContext};
mod predecessor;
mod producer_cascade;
mod route_completion;
pub(crate) use producer_cascade::CanonicalProducerRetirement;
mod relation_failure;
mod scope_dependents;
use relation_failure::relation_context;
pub(super) use relation_failure::relation_stop;
mod retained_origin;
use retained_origin::compare_retained_origin;

impl CanonicalCommitEnvelopeBody {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn install_required_output_origin(
        &mut self,
        execution: &ExecutionRequest<'_, '_>,
        runtime: &crate::runtime::RelationalPreparationRuntime,
        basis: &AdmittedRelationalBranchBasis,
        created: &CommitCreatedEntityBindings,
        created_relations: &CommitCreatedRelationBindings,
        state: &WorkingState,
        admission: crate::mvcc::RequiredOutputPublicationAdmission,
    ) -> Result<crate::history::data::RequiredOutputPublicationPositions, TransactionCommitError>
    {
        admission
            .installed
            .demand_source_scope(execution, basis, &admission.publication)?;
        let runtime = &crate::history::data::canonical_commit_envelope::required_output_origin::history_request::PreparationRuntimeRequest::new(runtime, execution);
        let result = execution
            .in_scope(|lease| {
                worth_execution::ExecutionWorkCeiling::new(execution.work_ceiling())
                    .run_with_checkpoint(lease, None, |context| {
                        self.install_completion(
                            runtime,
                            basis,
                            created,
                            created_relations,
                            state,
                            admission,
                            lease,
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
    pub(super) fn install_completion(
        &mut self,
        runtime: &crate::history::data::canonical_commit_envelope::required_output_origin::history_request::PreparationRuntimeRequest<'_, '_, '_>,
        basis: &AdmittedRelationalBranchBasis,
        created: &CommitCreatedEntityBindings,
        created_relations: &CommitCreatedRelationBindings,
        state: &WorkingState,
        admission: crate::mvcc::RequiredOutputPublicationAdmission,
        lease: Option<&worth_execution::ExecutionResourceLease<'_>>,
        context: &mut MapKernelContext<'_, '_>,
    ) -> Result<crate::history::data::RequiredOutputPublicationPositions, TransactionCommitError>
    {
        let crate::mvcc::RequiredOutputPublicationAdmission {
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
            || self.branch_context != *basis.descriptor().branch_id()
            || !self.is_unmerged_output_publication_step()
            || self.schema_transition.is_some()
            || self.schema_continuation_descriptor.is_some()
            || self.schema_reconciliation_descriptor.is_some()
        {
            return Err(crate::mvcc::origin_denial(
                "required-output installation changed before native preparation",
            ));
        }
        if let Publication::AuthoredCascadeRestoration {
            admission,
            restoration,
        } = publication
        {
            return super::authored_restoration::install(
                self,
                runtime,
                basis,
                created,
                created_relations,
                state,
                installed,
                admission,
                restoration,
                lease,
                context,
            );
        }
        if let Publication::AuthoredCascadeRetirement {
            admission,
            predecessor,
        } = publication
        {
            return super::authored_cascade::install(
                self,
                runtime,
                basis,
                created,
                created_relations,
                state,
                installed,
                admission,
                predecessor,
                lease,
                context,
            );
        }
        let (producer, dependency_identity) =
            publication.lineage_attribution().ok_or_else(|| {
                crate::mvcc::origin_denial(
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
                    self,
                    runtime,
                    basis,
                    predecessor,
                    meaning,
                    lease,
                    context,
                )?;
                (meaning, roles, Some(origin))
            }
            Publication::AuthoredCascadeRetirement { .. }
            | Publication::AuthoredCascadeRestoration { .. } => {
                unreachable!("cascade consumed before ordinary installation")
            }
            Publication::AuthoredRetirement {
                predecessor, roles, ..
            } => {
                let meaning = &predecessor.origin.meaning;
                let origin = predecessor::validate(
                    self,
                    runtime,
                    basis,
                    predecessor,
                    meaning,
                    lease,
                    context,
                )?;
                super::source_scope_ownership::require_cascade(
                    origin,
                    self.authored_cascade_retirement.as_deref(),
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
                self,
                runtime,
                &installed,
                prior,
                roles,
                succession,
                generator.authority(),
                context,
            )?
        {
            return self.install_producer_cascade(
                runtime,
                basis,
                created,
                created_relations,
                state,
                &installed,
                route_completion::RouteCompletion {
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
                context,
            );
        }
        if let Some(prior) = previous
            && !matches!(publication, Publication::Preserve { .. })
        {
            scope_dependents::require_retained_sources(
                self, runtime, &installed, prior, roles, context,
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
        self.install_route(
            runtime,
            basis,
            created,
            created_relations,
            state,
            &installed,
            route_completion::RouteCompletion {
                producer,
                dependency_identity,
                meaning,
                roles,
                previous,
                completion,
            },
            lease,
            context,
        )
    }
}
