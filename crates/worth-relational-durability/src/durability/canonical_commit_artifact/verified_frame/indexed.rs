//! Admit the exact framed body against its installed commit-index row.
use worth_relational_branch::internal_port::history::data::canonical_commit_envelope::CanonicalCommitEnvelopeBodyOwnerPort as _;
use super::VerifiedCanonicalCommitFrame;
use branch::history::commit::index_entry::RelationalCommitIndexEntry;
use branch::history::data::positioned_canonical_commit::PositionedCanonicalCommit;
use contracts::durability::canonical_commit_artifact::readmission_refusal::RelationalCanonicalCommitArtifactRefusal as Refusal;
use contracts::durability::data::recovery_stop::RelationalRecoveryStop;
use contracts::storage::data::storage_unavailable::{
    RelationalStorageSubject, RelationalStorageUnavailable,
    RelationalStorageUnavailableCause as Cause,
};
use worth_execution::ExecutionRequest;
use worth_relational_branch::internal_port as branch;
use worth_relational_contracts::internal_port as contracts;

impl VerifiedCanonicalCommitFrame {
    pub(crate) fn readmit_indexed(
        self,
        entry: &RelationalCommitIndexEntry,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<PositionedCanonicalCommit, RelationalRecoveryStop> {
        self.check_index(entry)?;
        let admitted = self
            .readmitted_body(execution)
            .map_err(|error| match error {
                Refusal::Request(cause) => RelationalRecoveryStop::Request(cause),
                Refusal::Content(cause) => cause.into(),
                Refusal::FieldText(cause) => {
                    contracts::symbols::data::field_text::field_text_recovery_stop(cause)
                }
                _ => RelationalStorageUnavailable {
                    subject: RelationalStorageSubject::Envelope(entry.position),
                    cause: Cause::Damaged,
                }
                .into(),
            })?;
        Self::check_positioned(entry, admitted, execution)
    }
    pub(crate) fn check_index(
        &self,
        entry: &RelationalCommitIndexEntry,
    ) -> Result<(), RelationalRecoveryStop> {
        let failure = |cause| RelationalStorageUnavailable {
            subject: RelationalStorageSubject::Envelope(entry.position),
            cause,
        };
        if self.position() != entry.position
            || self.commit_id() != entry.commit.commit_id
            || self.payload_digest().as_bytes() != &entry.payload_digest
        {
            return Err(failure(Cause::IntegrityMismatch).into());
        }
        Ok(())
    }
    fn check_positioned(
        entry: &RelationalCommitIndexEntry,
        admitted: PositionedCanonicalCommit,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<PositionedCanonicalCommit, RelationalRecoveryStop> {
        let failure = |cause| RelationalStorageUnavailable {
            subject: RelationalStorageSubject::Envelope(entry.position),
            cause,
        };
        if admitted.commit != entry.commit
            || admitted.envelope().association_discovery() != entry.association_discovery
            || admitted.schema_version != entry.schema_version
            || entry.root.schema_root()
                != &contracts::schema::data::authority_snapshot::schema_authority_snapshot_digest_bytes(
                    &admitted.schema_authority,
                )
        {
            return Err(failure(Cause::IntegrityMismatch).into());
        }
        contracts::execution::request_custody::check_request(execution)
            .map_err(RelationalRecoveryStop::Request)?;
        Ok(admitted)
    }
}
