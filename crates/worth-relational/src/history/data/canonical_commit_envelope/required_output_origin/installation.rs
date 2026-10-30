//! Pre-effect native membership installation and exact predecessor write fences.
use super::*;
use crate::mvcc::RequiredOutputPublication as Publication;
use crate::storage::overlay::WorkingState;
use crate::transactions::data::{CommitCreatedEntityBindings, CommitCreatedRelationBindings};
use worth_execution::{ExecutionRequest, MapKernelContext};
mod predecessor;
mod retained_origin;
use retained_origin::compare_retained_origin;

impl CanonicalCommitEnvelope {
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
    fn install_completion(
        &mut self,
        runtime: &crate::runtime::RelationalPreparationRuntime,
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
        context.checkpoint(1).map_err(origin_request)?;
        if installed.runtime_instance_id != runtime.runtime_instance_id()
            || installed.custom_invariant_generation
                != runtime.schema_contract_runtime.custom_invariant_generation
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
        let (producer, dependency_identity) = publication.lineage_attribution();
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
            Publication::AuthoredRetirement {
                predecessor, roles, ..
            } => {
                let meaning = &predecessor
                    .envelope
                    .required_output_origin
                    .as_ref()
                    .expect("Native custody retains canonical membership")
                    .meaning;
                let origin = predecessor::validate(
                    self,
                    runtime,
                    basis,
                    predecessor,
                    meaning,
                    lease,
                    context,
                )?;
                if roles
                    .iter()
                    .any(|role| role.posture != crate::mvcc::RequiredOutputRolePosture::Retire)
                    || roles.len()
                        != origin
                            .members
                            .iter()
                            .filter(|member| {
                                member.posture != crate::mvcc::RequiredOutputRolePosture::Retire
                            })
                            .count()
                {
                    return Err(crate::mvcc::origin_denial("authored retirement requires every prior live member and only Retire roles"));
                }
                (meaning, roles, Some(origin))
            }
        };
        if roles.is_empty() {
            return Err(crate::mvcc::origin_denial(
                "empty producer publication membership",
            ));
        }
        context
            .checkpoint(
                (roles.len() as u64)
                    .checked_mul(previous.map_or(1, |prior| {
                        u64::from(usize::BITS - prior.members.len().leading_zeros()) + 1
                    }))
                    .ok_or_else(|| {
                        origin_request(
                            crate::execution::RelationalExecutionDenialCause::WorkCounterOverflow,
                        )
                    })?,
            )
            .map_err(origin_request)?;
        let extra = roles
            .iter()
            .try_fold(0u64, |sum, role| {
                sum.checked_add(
                    (std::mem::size_of::<CanonicalOutputMember>()
                        + role.role.len()
                        + role.entity_name.len()
                        + previous
                            .and_then(|prior| match &role.target {
                                crate::transactions::data::EntityReference::Existing(id) => prior
                                    .members
                                    .binary_search_by_key(id, |member| member.id)
                                    .ok()
                                    .map(|index| &prior.members[index]),
                                _ => None,
                            })
                            .map_or(producer.len(), |member| member.birth_producer.len()))
                        as u64,
                )
            })
            .and_then(|n| n.checked_add(meaning.family.len() as u64))
            .and_then(|n| {
                n.checked_add(
                    previous.map_or(producer.len(), |prior| prior.birth_producer.len()) as u64,
                )
            })
            .ok_or_else(|| {
                origin_request(
                    crate::execution::RelationalExecutionDenialCause::ChargedBytesOverflow,
                )
            })?;
        let bytes = origin_retention_bytes(
            self,
            basis,
            producer.len(),
            roles.len(),
            created_relations
                .iter()
                .len()
                .checked_add(previous.map_or(0, |prior| prior.relations.len()))
                .ok_or_else(|| {
                    origin_request(
                        crate::execution::RelationalExecutionDenialCause::ChargedBytesOverflow,
                    )
                })?,
        )?
        .checked_add(extra)
        .ok_or_else(|| {
            origin_request(crate::execution::RelationalExecutionDenialCause::ChargedBytesOverflow)
        })?;
        context.checkpoint(bytes).map_err(origin_request)?;
        let memory = worth_execution::ExecutionMemoryReservation::reserve_in_scope(lease, bytes)
            .map_err(|stop| origin_request(crate::execution::denial::lease_denial(stop)))?;
        let members = super::members::capture(
            self,
            &runtime.history,
            basis,
            state,
            created,
            roles,
            previous,
            producer,
            dependency_identity,
            context,
        )?;
        let outputs = members.iter().map(|member| member.id).collect::<Vec<_>>();
        let mut relations = relations::capture_created_relations(
            self,
            created,
            created_relations,
            state,
            &outputs,
            previous.map_or(0, |prior| prior.relations.len()),
            lease,
            context,
        )?;
        if let Some(prior) = previous {
            relations::continue_relations(
                self,
                &runtime.history,
                prior,
                basis,
                state,
                &mut relations,
                context,
            )?;
        }
        let mut positions = basis.inner.root.output_publication_positions.clone();
        positions.insert(
            meaning,
            crate::history::data::RequiredOutputRoutePosition::Published(self.commit.commit_id),
            lease,
            context,
        )?;
        // Recorded in the canonical frame, like the envelope's branch cell (D11).
        let recorded_basis = basis.descriptor().in_canonical_frame().map_err(|denial| {
            crate::mvcc::origin_denial(format!(
                "output basis cannot be recorded in the canonical frame: {denial:?}"
            ))
        })?;
        self.required_output_origin = Some(std::sync::Arc::new(CanonicalRequiredOutputOrigin {
            commit: self.commit.clone(),
            basis: recorded_basis,
            producer: producer.to_owned(),
            dependency_identity,
            outputs,
            relations,
            meaning: meaning.clone(),
            members,
            publication: previous.map_or_else(
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
            ),
            birth: previous.map_or(self.commit.commit_id, |prior| prior.birth),
            birth_producer: previous
                .map_or_else(|| producer.to_owned(), |prior| prior.birth_producer.clone()),
            birth_dependency: previous.map_or(dependency_identity, |prior| prior.birth_dependency),
            issuer: Some(installed.authority.clone()),
            _funding: Some(memory),
        }));
        crate::history::data::publication_relations::validate(
            &basis.inner.root,
            Some(state),
            self.required_output_origin.as_deref().unwrap(/* Proof: This method assigns Some(required_output_origin) immediately above and borrows it without an intervening take or mutation. */ ),
            Some(self),
            &mut |commit| runtime.history.canonical_envelope(commit),
            &mut |version| runtime.history.canonical_envelope_for_version(version),
            runtime.history.canonical_catalog_len(),
            lease,
            context,
        )
        .map_err(|stop| relation_context(stop, "candidate", basis, self))?;
        if matches!(publication, Publication::AuthoredRetirement { .. })
            && self
                .required_output_origin
                .as_ref()
                .is_some_and(|origin| !origin.relations.is_empty())
        {
            return Err(crate::mvcc::origin_denial(
                "authored retirement must actually retire every owned relation",
            ));
        }
        Ok(positions)
    }
}

pub(super) fn relation_stop(
    stop: crate::history::data::CodecRetirementClosureStop,
) -> TransactionCommitError {
    match stop {
        crate::history::data::CodecRetirementClosureStop::Request(cause) => origin_request(cause),
        crate::history::data::CodecRetirementClosureStop::Payload(detail) => {
            crate::mvcc::origin_denial(detail)
        }
    }
}

fn relation_context(
    stop: crate::history::data::CodecRetirementClosureStop,
    stage: &str,
    basis: &AdmittedRelationalBranchBasis,
    candidate: &CanonicalCommitEnvelope,
) -> TransactionCommitError {
    match stop {
        crate::history::data::CodecRetirementClosureStop::Request(cause) => origin_request(cause),
        crate::history::data::CodecRetirementClosureStop::Payload(detail) => {
            crate::mvcc::origin_denial(format!(
                "{detail}; publication_stage={stage}; before={:?}; candidate={:?}",
                basis.descriptor(),
                candidate.commit
            ))
        }
    }
}
