//! Runtime custody is issued from sealed performed results or cold native readmission.
use crate::execution::request_stage::checkpoint;
use crate::history::data::CanonicalCommitEnvelope;
use std::sync::Arc;
mod current;
mod prepared;
pub(crate) use prepared::prepare;
pub use prepared::{RelationalPerformedOutputPublications, RelationalPreparedOutputPublications};

#[derive(Clone, Debug)]
pub struct RelationalRequiredOutputPublication {
    pub(crate) runtime_instance_id: u64,
    pub(crate) evidence: crate::publication::ownership::PublicationEvidence,
    pub(crate) origin: Arc<crate::history::data::CanonicalRequiredOutputOrigin>,
    pub(crate) authority: worth_proof::RequiredOutputPublicationAuthority,
    pub(crate) readmitted_at: Option<crate::branch::AdmittedRelationalBranchBasis>,
}

impl RelationalRequiredOutputPublication {
    pub fn producer(&self) -> &str {
        &self.origin.producer
    }
    pub fn meaning(&self) -> &super::RequiredOutputPublicationMeaning {
        &self.origin.meaning
    }
    /// Actual performed roles, including retired members, in canonical native order.
    pub fn members(
        &self,
    ) -> impl ExactSizeIterator<
        Item = (
            &str,
            &str,
            super::RequiredOutputRolePosture,
            crate::identity::data::EntityId,
        ),
    > {
        self.origin.members.iter().map(|member| {
            (
                member.role.as_str(),
                member.entity_name.as_str(),
                member.posture,
                member.id,
            )
        })
    }
    /// A genuine Native correction origin, independent of a producer execution locator.
    pub fn is_authored_restoration(&self) -> bool {
        matches!(
            self.origin.publication,
            crate::history::data::CanonicalOutputPublication::AuthoredRestoration { .. }
        )
    }
    /// This Native origin selects the producer's own retained record identities.
    pub fn is_historical_selection(&self) -> bool {
        matches!(
            self.origin.publication,
            crate::history::data::CanonicalOutputPublication::HistoricalSelection { .. }
        )
    }
    pub fn authority(&self) -> &worth_proof::RequiredOutputPublicationAuthority {
        &self.authority
    }
    /// Whether both name one canonical Native origin of this runtime: the same
    /// publishing commit, producer, dependency and complete route meaning.
    pub fn same_origin(&self, other: &Self) -> bool {
        self.runtime_instance_id == other.runtime_instance_id
            && self.origin.commit == other.origin.commit
            && self.origin.producer == other.origin.producer
            && self.origin.dependency_identity == other.origin.dependency_identity
            && self.origin.meaning == other.origin.meaning
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
            evidence: crate::publication::ownership::PublicationEvidence::Canonical(Arc::clone(
                envelope,
            )),
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
    /// Issue each genuine route's own ordinal seal from this performed commit.
    /// Iteration funds only the reached origins under the original request.
    pub fn required_output_publications<'a>(
        &'a self,
        installed: &'a super::RelationalInstalledApplicationPublicationAuthority,
        execution: &'a worth_execution::ExecutionRequest<'_, '_>,
    ) -> impl Iterator<
        Item = Result<
            RelationalRequiredOutputPublication,
            crate::transactions::data::TransactionCommitError,
        >,
    > + 'a {
        let envelope = &self.publication().envelope;
        envelope
            .output_origins()
            .enumerate()
            .filter_map(move |(ordinal, origin)| {
                let selected = execution
                    .run_native_with_checkpoint(|context| {
                        // Reaching a foreign origin still consumes traversal work.
                        checkpoint(context, 1).map_err(execution_stop)?;
                        if self.snapshot.runtime_instance_id() != installed.runtime_instance_id
                            || !origin.issuer.as_ref().is_some_and(|issuer| {
                                issuer.same_publication_owner(&installed.authority)
                            })
                        {
                            return Ok(None);
                        }
                        Ok(Some(RelationalRequiredOutputPublication {
                            runtime_instance_id: installed.runtime_instance_id,
                            evidence: crate::publication::ownership::PublicationEvidence::Canonical(
                                Arc::clone(envelope),
                            ),
                            origin: Arc::clone(origin),
                            readmitted_at: None,
                            authority: installed.authority.seal_required_output_publication_at(
                                envelope.commit.commit_id.0,
                                ordinal,
                            ),
                        }))
                    })
                    .map_err(execution_stop)
                    .and_then(|answer| answer.0);
                match selected {
                    Ok(None) => None,
                    Ok(Some(publication)) => Some(Ok(publication)),
                    Err(cause) => Some(Err(cause)),
                }
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
        if installed
            .source_scope_ownership
            .scope_kind(producer)
            .is_some()
        {
            installed.require_current_ownership_storage(basis, meaning.source_scope)?;
        }
        if installed
            .source_scope_ownership
            .scope_kind(producer)
            .is_some()
            && roles
                .iter()
                .any(|role| role.posture != super::RequiredOutputRolePosture::Retire)
        {
            return current::readmit(
                execution, installed, basis, producer, dependency, meaning, roles,
            );
        }
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
            evidence,
            origin,
            ordinal,
        } = selected;
        if origin.source_scope_owned {
            installed.require_current_ownership_storage(basis, origin.meaning.source_scope)?;
        }
        if crate::history::data::readmit_required_output_route(execution, self, basis, meaning)
            .map_err(cold_stop)?
            != Some(
                crate::history::data::RequiredOutputRoutePosition::Published(
                    origin.commit.commit_id,
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
            .seal_required_output_publication_at(origin.commit.commit_id.0, ordinal);
        Ok(RelationalRequiredOutputPublication {
            runtime_instance_id: installed.runtime_instance_id,
            evidence,
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

impl crate::transactions::data::CommitResult {
    /// Descriptive roles of this commit's canonical primary publication. Closure
    /// witnesses used to admit a succession are not members of that publication.
    /// This view grants no publication or succession custody.
    pub fn primary_output_members(
        &self,
    ) -> Option<
        impl ExactSizeIterator<
            Item = (
                &str,
                &str,
                super::RequiredOutputRolePosture,
                crate::identity::data::EntityId,
            ),
        >,
    > {
        self.publication()
            .envelope
            .required_output_origin
            .as_ref()
            .map(|origin| {
                origin
                    .members
                    .iter()
                    .map(|m| (m.role.as_str(), m.entity_name.as_str(), m.posture, m.id))
            })
    }
}
