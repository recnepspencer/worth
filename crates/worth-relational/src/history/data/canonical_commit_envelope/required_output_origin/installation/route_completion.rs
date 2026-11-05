//! Capture one authenticated route; both ordinary and grouped publication use this path.
use super::*;
use crate::mvcc::{RequiredOutputPublicationMeaning, RequiredOutputRoleTarget};
pub(super) struct RouteCompletion<'a> {
    pub(super) producer: &'a str,
    pub(super) dependency_identity: [u8; 32],
    pub(super) meaning: &'a RequiredOutputPublicationMeaning,
    pub(super) roles: &'a [RequiredOutputRoleTarget],
    pub(super) previous: Option<&'a CanonicalRequiredOutputOrigin>,
    pub(super) completion: CanonicalOutputPublication,
}
impl CanonicalCommitEnvelopeBody {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn install_route(
        &mut self,
        runtime: &crate::runtime::RelationalPreparationRuntime,
        basis: &AdmittedRelationalBranchBasis,
        created: &CommitCreatedEntityBindings,
        created_relations: &CommitCreatedRelationBindings,
        state: &WorkingState,
        installed: &crate::mvcc::RelationalInstalledApplicationPublicationAuthority,
        route: RouteCompletion<'_>,
        lease: Option<&worth_execution::ExecutionResourceLease<'_>>,
        context: &mut MapKernelContext<'_, '_>,
    ) -> Result<crate::history::data::RequiredOutputPublicationPositions, TransactionCommitError>
    {
        let RouteCompletion {
            producer,
            dependency_identity,
            meaning,
            roles,
            previous,
            completion,
        } = route;
        let retiring = matches!(
            completion,
            CanonicalOutputPublication::AuthoredRetirement { .. }
                | CanonicalOutputPublication::ProducerRetirement { .. }
        );
        super::source_scope_ownership::validate(
            basis,
            installed,
            producer,
            meaning,
            previous,
            self.commit.commit_id,
            state,
            retiring,
            context,
        )?;
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
            source_scope_owned: previous.map_or_else(
                || {
                    installed
                        .source_scope_ownership
                        .scope_kind(producer)
                        .is_some()
                },
                |prior| prior.source_scope_owned,
            ),
            producer: producer.to_owned(),
            dependency_identity: worth_foundational::CanonicalDigestId::new(dependency_identity),
            outputs,
            relations,
            meaning: meaning.clone(),
            members,
            publication: completion,
            birth: previous.map_or(self.commit.commit_id, |prior| prior.birth),
            birth_producer: previous
                .map_or_else(|| producer.to_owned(), |prior| prior.birth_producer.clone()),
            birth_dependency: previous.map_or(
                worth_foundational::CanonicalDigestId::new(dependency_identity),
                |prior| prior.birth_dependency,
            ),
            issuer: Some(installed.authority.clone()),
            _funding: Some(memory),
            _cascade_memory: None,
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
        if retiring
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
