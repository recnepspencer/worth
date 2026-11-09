//! Judge executed application effects before any new publication can be adopted.
use super::PreparedRelationalCommitCandidate;
use crate::mvcc::{RelationalRequiredOutputPublication, RequiredOutputRolePosture};
use crate::transactions::data::TransactionCommitError;

/// The exact prepared candidate preserves this current Native publication.
/// It carries the original publication; it is never a newly performed origin.
#[derive(Debug)]
pub struct RelationalCurrentOutputAdoption {
    publication: RelationalRequiredOutputPublication,
    basis: crate::branch::RelationalBranchBasisDescriptor,
    selected: std::sync::Arc<crate::history::data::CanonicalRequiredOutputOrigin>,
}

impl RelationalCurrentOutputAdoption {
    pub fn publication(&self) -> &RelationalRequiredOutputPublication {
        &self.publication
    }

    pub fn dependency_identity(&self) -> &[u8; 32] {
        &self.selected.dependency_identity
    }

    pub fn meaning(&self) -> &crate::mvcc::RequiredOutputPublicationMeaning {
        &self.selected.meaning
    }

    /// The executed completion's admitted live roles, with original identities.
    /// These describe the adoption, never a newly performed publication.
    pub fn members(
        &self,
    ) -> impl ExactSizeIterator<
        Item = (
            &str,
            &str,
            RequiredOutputRolePosture,
            crate::identity::data::EntityId,
        ),
    > {
        self.selected
            .members
            .iter()
            .map(|m| (m.role.as_str(), m.entity_name.as_str(), m.posture, m.id))
    }

    pub fn basis(&self) -> &crate::branch::RelationalBranchBasisDescriptor {
        &self.basis
    }
}

impl PreparedRelationalCommitCandidate {
    pub(in crate::mvcc::publication) fn omitted_appendix_publication_refusal(
        &self,
    ) -> Option<crate::mvcc::RelationalPublicationFailureKind> {
        let payload = self
            ._payload
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let prior = payload.as_ref()?.execution.appendix_adoption()?;
        Some(
            crate::mvcc::RelationalPublicationFailureKind::OutputAdoptionRequired {
                source: prior.meaning.source_scope,
                producer: prior.producer.clone(),
                family: prior.meaning.family.clone(),
            },
        )
    }

    /// All ordinary command, invariant, route, write-position and ownership
    /// admission has completed. Only an unchanged application can retain its
    /// predecessor rather than publish the prepared current-ownership rows.
    pub fn prepare_current_output_adoption(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        basis: &crate::branch::AdmittedRelationalBranchBasis,
        previous: &RelationalRequiredOutputPublication,
    ) -> Result<Option<RelationalCurrentOutputAdoption>, TransactionCommitError> {
        execution
            .run_native_with_checkpoint(|context| {
                crate::execution::request_stage::checkpoint(context, 1).map_err(request_stop)?;
                let payload = self
                    ._payload
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                let payload = payload.as_ref().ok_or_else(|| {
                    crate::mvcc::origin_denial("executed adoption candidate is unavailable")
                })?;
                if self.lifetime_expired()
                    || !basis.is_current()
                    || basis.descriptor() != &payload.expected_basis
                    || previous.runtime_instance_id != self.runtime_instance_id
                    || !payload.execution.application_unchanged()
                {
                    return Ok(None);
                }
                let envelope = payload.execution.canonical_envelope();
                let Some(next) = envelope.required_output_origin.as_ref() else {
                    return Ok(None);
                };
                crate::execution::request_stage::checkpoint(
                    context,
                    next.producer
                        .len()
                        .saturating_add(previous.origin.producer.len())
                        .saturating_add(next.meaning.family.len())
                        .saturating_add(previous.origin.meaning.family.len())
                        .saturating_add(16) as u64,
                )
                .map_err(request_stop)?;
                if !matches!(
                    next.publication,
                    crate::history::data::CanonicalOutputPublication::Preserve { .. }
                ) || envelope.output_origins().nth(1).is_some()
                    || next.publication.predecessor() != Some(previous.origin.commit.commit_id)
                    || next.producer != previous.origin.producer
                    || next.source_scope_owned != previous.origin.source_scope_owned
                    || !next.meaning.same_route(&previous.origin.meaning)
                    || next.meaning.implementation != previous.origin.meaning.implementation
                    || !next.issuer.as_ref().is_some_and(|issuer| {
                        issuer.owns_required_output_publication(&previous.authority)
                    })
                {
                    return Ok(None);
                }
                crate::execution::request_stage::checkpoint(
                    context,
                    previous.origin.members.len() as u64,
                )
                .map_err(request_stop)?;
                let mut prior = previous
                    .origin
                    .members
                    .iter()
                    .filter(|member| member.posture != RequiredOutputRolePosture::Retire);
                for member in &next.members {
                    crate::execution::request_stage::checkpoint(
                        context,
                        (member.role.len() + member.entity_name.len() + 2) as u64,
                    )
                    .map_err(request_stop)?;
                    let Some(old) = prior.next() else {
                        return Ok(None);
                    };
                    if member.posture != RequiredOutputRolePosture::Preserve
                        || member.id != old.id
                        || member.kind != old.kind
                        || member.role != old.role
                        || member.entity_name != old.entity_name
                        || member.birth != old.birth
                        || member.birth_producer != old.birth_producer
                        || member.birth_dependency != old.birth_dependency
                        || member.before_write != Some(old.after_write)
                    {
                        return Ok(None);
                    }
                }
                if prior.next().is_some() {
                    return Ok(None);
                }
                Ok(Some(RelationalCurrentOutputAdoption {
                    publication: previous.clone(),
                    basis: payload.expected_basis.clone(),
                    selected: std::sync::Arc::clone(next),
                }))
            })
            .map_err(request_stop)?
            .0
    }
}

fn request_stop(
    cause: impl Into<crate::execution::RelationalExecutionDenialCause>,
) -> TransactionCommitError {
    TransactionCommitError::execution(crate::transactions::data::CommitExecutionDenial {
        kind: crate::transactions::data::CommitExecutionDenialKind::Cause(cause.into()),
        partition_identity: None,
    })
}
