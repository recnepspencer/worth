//! Current frame and owner payload admission under the invoking request.
use super::body_read::complete;
use super::RelationalCanonicalCommitArtifact as Artifact;
use crate::durability::migration::ReadmittedCanonicalCommit;
use contracts::durability::canonical_commit_artifact::frame::CanonicalCommitArtifactHeader as Header;
use contracts::durability::canonical_commit_artifact::payload_digest::RelationalCanonicalCommitArtifactDigestOwnerPort as _;
use contracts::durability::canonical_commit_artifact::readmission_refusal::RelationalCanonicalCommitArtifactRefusal as Refusal;
use worth_execution::{ExecutionImmutableBytes, ExecutionRequest, MapKernelContext};
use worth_relational_contracts::internal_port as contracts;

impl Artifact {
    /// Admit current bytes only: liveness, bounded frame, funded integrity,
    /// original allocation quotes, controlled content/schema decoding, header
    /// agreement, actual positioned meaning, and funded canonical comparison.
    pub fn readmit(
        bytes: &[u8],
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<ReadmittedCanonicalCommit, Refusal> {
        Self::readmit_with_input(bytes, execution, None)
    }

    pub(super) fn readmit_with_input(
        bytes: &[u8],
        execution: &ExecutionRequest<'_, '_>,
        input: Option<ExecutionImmutableBytes>,
    ) -> Result<ReadmittedCanonicalCommit, Refusal> {
        contracts::execution::request_custody::check_request(execution)
            .map_err(Refusal::Request)?;
        let (header, payload) = Header::inspect(bytes)?;
        let mut input = input;
        let mut answer = None;
        let handoff = contracts::execution::request_stage::optional_stage(
            execution,
            |context, _| {
                answer = Some((|| {
                    let digest = contracts::durability::canonical_commit_artifact::payload_digest::RelationalCanonicalCommitArtifactDigest::of_payload_for_request(
                    header.position.0, header.commit_id.0, payload, context).map_err(digest_denial)?;
                    header.require_digest(digest)?;
                    Self::readmit_payload(&header, payload, execution, context, input.take())
                })());
            },
            |_| 0,
        );
        complete(handoff, answer)
    }

    pub fn readmit_framed(
        &self,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<ReadmittedCanonicalCommit, Refusal> {
        self.frame.readmitted_body(execution).map(|body| {
            ReadmittedCanonicalCommit::exact(body)
                .with_verified_journal_payload(*self.payload_digest().as_bytes())
        })
    }

    fn readmit_payload(
        header: &Header,
        payload: &[u8],
        execution: &ExecutionRequest<'_, '_>,
        context: &mut MapKernelContext<'_, '_>,
        input: Option<ExecutionImmutableBytes>,
    ) -> Result<ReadmittedCanonicalCommit, Refusal> {
        super::body_read::readmit_canonical_body(header, payload, execution, context, input).map(
            |body| {
                ReadmittedCanonicalCommit::exact(body)
                    .with_verified_journal_payload(*header.digest.as_bytes())
            },
        )
    }
}

fn digest_denial(
    cause: contracts::durability::canonical_commit_artifact::payload_digest::RequestDigestDenial,
) -> Refusal {
    match cause {
        contracts::durability::canonical_commit_artifact::payload_digest::RequestDigestDenial::Frame(cause) => Refusal::Frame(cause),
        contracts::durability::canonical_commit_artifact::payload_digest::RequestDigestDenial::Request(cause) => Refusal::Request(cause),
    }
}
