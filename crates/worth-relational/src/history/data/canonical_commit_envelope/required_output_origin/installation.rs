//! Pre-effect native membership installation and exact predecessor write fences.
use super::*;
use crate::mvcc::RequiredOutputPublication as Publication;
use crate::storage::overlay::WorkingState;
use crate::transactions::data::{CommitCreatedEntityBindings, CommitCreatedRelationBindings};
use worth_execution::{ExecutionRequest, MapKernelContext};
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
        let running = publication.running();
        if installed.runtime_instance_id != runtime.runtime_instance_id()
            || installed.custom_invariant_generation
                != runtime.schema_contract_runtime.custom_invariant_generation
            || installed.schema_digest != basis.inner.root.schema_authority().authority_digest()
            || !installed.authority.admits_required_output(running)
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
        let producer = running.producer();
        let dependency_identity = *running.dependency_identity();
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
                admission,
                predecessor,
                meaning,
                roles,
            } => {
                if predecessor.runtime_instance_id != runtime.runtime_instance_id()
                    || !installed
                        .authority
                        .admits_required_output_succession(admission, &predecessor.authority)
                {
                    return Err(crate::mvcc::origin_denial(
                        "foreign native predecessor publication custody",
                    ));
                }
                let origin = predecessor
                    .envelope
                    .required_output_origin
                    .as_ref()
                    .ok_or_else(|| {
                        crate::mvcc::origin_denial(
                            "predecessor lacks native publication membership",
                        )
                    })?;
                context
                    .checkpoint((origin.meaning.family.len() + meaning.family.len() + 96) as u64)
                    .map_err(origin_request)?;
                if origin.meaning.family != meaning.family
                    || origin.meaning.source_scope != meaning.source_scope
                    || origin.meaning.source_partition != meaning.source_partition
                {
                    return Err(crate::mvcc::origin_denial(
                        "successor changes complete installed family/source correspondence",
                    ));
                }
                let position = basis
                    .inner
                    .root
                    .output_publication_positions
                    .get(&origin.meaning, context)?;
                let exact_cold_basis = predecessor
                    .readmitted_at
                    .as_ref()
                    .is_some_and(|readmitted| readmitted.descriptor() == basis.descriptor());
                if position.is_some_and(|selected| {
                    selected
                        != crate::history::data::RequiredOutputRoutePosition::Published(
                            origin.commit.commit_id,
                        )
                }) || (position.is_none() && !exact_cold_basis)
                {
                    return Err(crate::mvcc::origin_denial(
                        "predecessor is not the publication selected at the native before image",
                    ));
                }
                // The carried holder is checked against its exact performed native coordinate,
                // not an ancestry search or a caller-supplied description.
                context.checkpoint(1).map_err(origin_request)?;
                let canonical = runtime
                    .history
                    .canonical_envelope_for_version(origin.commit.version_id)
                    .ok_or_else(|| {
                        crate::mvcc::origin_denial("retained predecessor canonical position absent")
                    })?;
                let expected = canonical.required_output_origin.as_deref().ok_or_else(|| {
                    crate::mvcc::origin_denial("retained predecessor native membership absent")
                })?;
                compare_retained_origin(origin, expected, context)?;
                canonical
                    .validate_output_membership_header()
                    .map_err(relation_stop)?;
                crate::history::data::publication_relations::validate(
                    &basis.inner.root,
                    None,
                    origin,
                    None,
                    &mut |commit| runtime.history.canonical_envelope(commit),
                    &mut |version| runtime.history.canonical_envelope_for_version(version),
                    runtime.history.canonical_catalog_len(),
                    lease,
                    context,
                )
                .map_err(relation_stop)?;
                (meaning, roles, Some(origin.as_ref()))
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
        self.required_output_origin = Some(std::sync::Arc::new(CanonicalRequiredOutputOrigin {
            commit: self.commit.clone(),
            basis: basis.descriptor().clone(),
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
                |prior| CanonicalOutputPublication::Preserve {
                    predecessor: prior.commit.commit_id,
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
            self.required_output_origin.as_deref().unwrap(),
            Some(self),
            &mut |commit| runtime.history.canonical_envelope(commit),
            &mut |version| runtime.history.canonical_envelope_for_version(version),
            runtime.history.canonical_catalog_len(),
            lease,
            context,
        )
        .map_err(relation_stop)?;
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
