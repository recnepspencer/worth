//! Runtime custody is issued from sealed performed results or cold native readmission.
use crate::history::data::CanonicalCommitEnvelope;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct RelationalRequiredOutputPublication {
    pub(crate) runtime_instance_id: u64,
    pub(crate) envelope: Arc<CanonicalCommitEnvelope>,
    pub(crate) origin: Arc<crate::history::data::CanonicalRequiredOutputOrigin>,
    pub(crate) authority: worth_proof::RequiredOutputPublicationAuthority,
    pub(crate) readmitted_at: Option<crate::branch::AdmittedRelationalBranchBasis>,
}

impl RelationalRequiredOutputPublication {
    pub fn authority(&self) -> &worth_proof::RequiredOutputPublicationAuthority {
        &self.authority
    }
}

impl crate::transactions::data::CommitResult {
    pub fn required_output_publication(
        &self,
        installed: &super::RelationalInstalledApplicationPublicationAuthority,
    ) -> Option<RelationalRequiredOutputPublication> {
        let envelope = &self.publication().envelope;
        (self.snapshot.runtime_instance_id() == installed.runtime_instance_id
            && envelope
                .required_output_origin
                .as_ref()
                .and_then(|origin| origin.issuer.as_ref())
                .is_some_and(|issuer| issuer.same_publication_owner(&installed.authority)))
        .then(|| RelationalRequiredOutputPublication {
            runtime_instance_id: installed.runtime_instance_id,
            envelope: Arc::clone(envelope),
            origin: Arc::clone(
                envelope
                    .required_output_origin
                    .as_ref()
                    .expect("performed single publication"),
            ),
            readmitted_at: None,
            authority: installed
                .authority
                .seal_required_output_publication(envelope.commit.commit_id.0),
        })
    }
}

impl crate::runtime::RelationalRuntime {
    /// Explicit cold readmission for a restored correspondence. Warm callers
    /// retain the prior native holder and never traverse canonical history.
    pub fn readmit_required_output_publication(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        installed: &super::RelationalInstalledApplicationPublicationAuthority,
        basis: &crate::branch::AdmittedRelationalBranchBasis,
        producer: &str,
        dependency: &[u8; 32],
        meaning: &crate::mvcc::RequiredOutputPublicationMeaning,
        roles: &[crate::mvcc::RequiredOutputRoleTarget],
    ) -> Result<
        RelationalRequiredOutputPublication,
        crate::transactions::data::TransactionCommitError,
    > {
        if installed.runtime_instance_id != self.runtime_instance_id() {
            return Err(super::origin_denial(
                "foreign installed publisher for cold readmission",
            ));
        }
        let selected = crate::history::data::validate_codec_output_correspondence(
            execution, self, basis, producer, dependency, meaning, roles,
        )
        .map_err(cold_stop)?;
        let crate::history::data::SelectedOutputPublication {
            envelope,
            origin,
            ordinal,
        } = selected;
        if crate::history::data::readmit_required_output_route(execution, self, basis, meaning)
            .map_err(cold_stop)?
            != Some(
                crate::history::data::RequiredOutputRoutePosition::Published(
                    envelope.commit.commit_id,
                ),
            )
        {
            return Err(super::origin_denial("superseded complete family publication is historical association, not a current predecessor"));
        }
        if installed
            .source_scope_ownership
            .scope_kind(&origin.producer)
            .is_some()
            != origin.source_scope_owned
        {
            return Err(
                crate::transactions::data::TransactionCommitError::publication(
                    crate::publication::data::PublicationError::authored_retirement(
                        crate::publication::data::AuthoredRetirementRefusal::CorruptPublication,
                        crate::transactions::data::RecordRef::Entity(origin.meaning.source_scope),
                    ),
                ),
            );
        }
        if origin
            .issuer
            .as_ref()
            .is_some_and(|issuer| !issuer.same_publication_owner(&installed.authority))
        {
            return Err(super::origin_denial(
                "foreign installed owner cannot reissue live publication custody",
            ));
        }
        let authority = installed
            .authority
            .seal_required_output_publication_at(envelope.commit.commit_id.0, ordinal);
        Ok(RelationalRequiredOutputPublication {
            runtime_instance_id: installed.runtime_instance_id,
            envelope,
            origin,
            authority,
            readmitted_at: Some(basis.clone()),
        })
    }
}

pub(super) fn cold_stop(
    stop: crate::history::data::CodecRetirementClosureStop,
) -> crate::transactions::data::TransactionCommitError {
    match stop {
        crate::history::data::CodecRetirementClosureStop::Request(cause) => execution_stop(cause),
        crate::history::data::CodecRetirementClosureStop::Payload(detail) => {
            super::origin_denial(detail)
        }
    }
}

pub(super) fn execution_stop(
    cause: impl Into<crate::execution::RelationalExecutionDenialCause>,
) -> crate::transactions::data::TransactionCommitError {
    crate::transactions::data::TransactionCommitError::execution(
        crate::transactions::data::CommitExecutionDenial {
            kind: crate::transactions::data::CommitExecutionDenialKind::Cause(cause.into()),
            partition_identity: None,
        },
    )
}
