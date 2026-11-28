//! Freeze original producer DATA under the actual configuration/publication epoch.
use crate::durability::source_initial_inputs::producer_roster::{
    OriginalBranchSchemaInputs, OriginalBranchSchemaInputsDenial,
};
use crate::runtime::state::subsystems::durability::checkpoint_selection::RelationalCheckpointManifestSelection;
use contracts::branch::transition::sequence::RelationalBranchTransitionSequence;
use contracts::durability::data::native_checkpoint_capture_denial::{
    array_capture_stop as array_stop, capture_stop,
    RelationalNativeCheckpointCaptureDenial as Denial,
};
use contracts::history::data::commit_receipt::RelationalCommitReceipt;
use contracts::publication::patch::data::patch_position::PatchStreamPosition;
use std::sync::Arc;
use worth_execution::{
    ExecutionArray, ExecutionOrderedMap, ExecutionRequest, ExecutionRequestArrayBuilder,
};
use worth_relational_branch::internal_port::history::commit::artifact::original_inputs::CapturedOriginalCanonicalInputs;
use worth_relational_branch::internal_port::history::commit::artifact::RelationalCommitArtifact;
use worth_relational_branch::internal_port::history::data::canonical_commit_envelope::CanonicalCommitEnvelope;
use worth_relational_branch::internal_port::runtime::state::subsystems::history::HistorySubsystem;
use worth_relational_contracts::internal_port as contracts;
use worth_relational_storage::internal_port::runtime::state::subsystems::services::symbol_table::RuntimeSymbolTable;

pub struct FrozenOriginalInputs {
    pub base: Option<crate::durability::checkpoint_admission::FrozenSelectedCaptureBase>,
    pub(super) artifacts: ExecutionArray<Arc<RelationalCommitArtifact>>,
    pub(super) branches: OriginalBranchSchemaInputs,
    pub(super) symbols: contracts::symbols::data::string_interner::StringInterner,
    journal_source: Option<Arc<dyn worth_relational_storage::internal_port::durability::medium::cold_source::RelationalDurableColdSource>>,
    journal: Option<crate::durability::data::OriginalCheckpointRead>,
}

#[derive(Clone)]
pub(super) struct OriginalCanonicalCommit {
    pub(super) position: PatchStreamPosition,
    pub(super) artifact: Arc<RelationalCommitArtifact>,
    pub(super) envelope: Arc<CanonicalCommitEnvelope>,
    pub(super) inputs: CapturedOriginalCanonicalInputs,
    pub(super) original_frame: worth_execution::ExecutionImmutableBytes,
    pub(super) original_payload_digest: [u8; 32],
}

impl FrozenOriginalInputs {
    /// Called while the actual complete capture still holds both epochs.
    pub fn capture(
        history: &HistorySubsystem,
        capture_branch_schema: impl FnOnce() -> Result<
            OriginalBranchSchemaInputs,
            OriginalBranchSchemaInputsDenial,
        >,
        selection: &RelationalCheckpointManifestSelection,
        mut journal_source: Option<Arc<dyn worth_relational_storage::internal_port::durability::medium::cold_source::RelationalDurableColdSource>>,
        symbols: &RuntimeSymbolTable,
        receipt: Option<&RelationalCommitReceipt>,
        covered_transitions: RelationalBranchTransitionSequence,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, Denial> {
        let base = selection.freeze_capture_base()?;
        let branches = capture_branch_schema().map_err(|cause| match cause {
            OriginalBranchSchemaInputsDenial::Request(cause) => Denial::Request(cause.into()),
            OriginalBranchSchemaInputsDenial::SizeOverflow => Denial::Request(
                contracts::execution::denial_cause::RelationalExecutionDenialCause::ChargedBytesOverflow,
            ),
            OriginalBranchSchemaInputsDenial::OriginalPrefixMismatch => {
                Denial::OriginalAuthorityUnavailable
            }
        })?;
        if branches.birth.is_none() {
            return Err(Denial::OriginalAuthorityUnavailable);
        }
        let mut ordered = ExecutionOrderedMap::default();
        let catalog = history
            .commit_artifacts_for_request(execution)
            .map_err(array_stop)?;
        for artifact in catalog.iter() {
            if artifact.original_inputs().is_none() {
                return Err(Denial::OriginalAuthorityUnavailable);
            }
            let cell = artifact.envelope_residency();
            let position = cell
                .captured_position()
                .ok_or(Denial::OriginalAuthorityUnavailable)?;
            if ordered.get(&position).is_some() {
                return Err(Denial::OriginalAuthorityUnavailable);
            }
            if let Some((_, source)) = cell.journal_placement() {
                if journal_source
                    .as_ref()
                    .is_some_and(|prior| !Arc::ptr_eq(prior, &source))
                {
                    return Err(Denial::OriginalAuthorityUnavailable);
                }
                journal_source = Some(source);
            }
            ordered
                .insert_cloned_native_for_request(&position, artifact, *execution)
                .map_err(map_stop)?;
        }
        let mut artifacts = ExecutionRequestArrayBuilder::allocate(*execution, ordered.len())
            .map_err(array_stop)?;
        for (offset, (position, artifact)) in ordered.iter().enumerate() {
            if position.0 != offset as u64 + 1 {
                return Err(Denial::OriginalAuthorityUnavailable);
            }
            artifacts.push(artifact.clone()).map_err(array_stop)?;
        }
        let artifacts = artifacts.seal().map_err(array_stop)?;
        let same_receipt = match (artifacts.last(), receipt) {
            (None, None) => true,
            (Some(artifact), Some(receipt)) => {
                artifact.commit_id() == receipt.commit_id
                    && artifact.version_id() == receipt.version_id
                    && artifact.identity().authoring_branch() == &receipt.branch_id
                    && artifact.parentage().as_slice() == receipt.parents.as_slice()
            }
            _ => false,
        };
        if !same_receipt
            || branches
                .transitions()
                .last()
                .map_or(0, |entry| entry.positioned.sequence().0)
                != covered_transitions.0
        {
            return Err(Denial::OriginalAuthorityUnavailable);
        }
        Ok(Self {
            base,
            artifacts,
            branches,
            journal_source,
            journal: None,
            symbols: symbols.interner_snapshot(),
        })
    }

    /// Physical acquisition is after BOTH logical snapshot epochs are released.
    /// A larger append-only N cut is consumed only at the frozen C/T positions.
    pub(super) fn protect_original_read(
        &mut self,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<(), Denial> {
        if let Some(source) = self.journal_source.take() {
            let protection = worth_relational_storage::internal_port::durability::medium::read_protection::RelationalDurableReadProtection::capture(
                &source, execution,
            )
            .map_err(|cause| {
                capture_stop(worth_relational_storage::internal_port::durability::medium::owned_read_stop::owned_stop(
                    cause,
                    contracts::storage::data::storage_unavailable::RelationalStorageSubject::Envelope(
                        contracts::publication::patch::data::patch_position::PatchStreamPosition(0),
                    ),
                ))
            })?;
            for artifact in self.artifacts.iter() {
                if let Some((entry, actual_source)) =
                    artifact.envelope_residency().journal_placement()
                    && (!Arc::ptr_eq(&actual_source, &source)
                        || entry.position > protection.bounds().canonical)
                {
                    return Err(Denial::OriginalAuthorityUnavailable);
                }
            }
            if let Some(accepted) = &self.branches.accepted_initial_inputs {
                let expected = accepted
                    .artifact
                    .as_ref()
                    .ok_or(Denial::OriginalAuthorityUnavailable)?;
                let actual = protection
                    .read_initial_inputs_owned(&source, execution, 1 << 30)
                    .map_err(|cause| {
                        capture_stop(worth_relational_storage::internal_port::durability::medium::owned_read_stop::owned_stop(
                            cause,
                            contracts::storage::data::storage_unavailable::RelationalStorageSubject::Envelope(
                                contracts::publication::patch::data::patch_position::PatchStreamPosition(0),
                            ),
                        ))
                    })?
                    .ok_or(Denial::OriginalAuthorityUnavailable)?;
                let actual = crate::durability::source_initial_inputs::RelationalSourceInitialInputsArtifact::from_owned(actual, execution)
                    .and_then(|frame| frame.readmit(execution)).map_err(initial_inputs_stop)?;
                if actual.entries.get(..accepted.entries.len()) != Some(&accepted.entries[..])
                    || actual.main != accepted.main
                    || (actual
                        .artifact
                        .as_ref()
                        .is_none_or(|frame| frame.as_bytes() != expected.as_bytes())
                        && !(accepted.revision == 1
                            && accepted.entries.len() == 1
                            && actual.revision == 2
                            && actual.predecessor == Some(expected.payload_digest())))
                {
                    return Err(Denial::OriginalAuthorityUnavailable);
                }
            }
            self.journal = Some((source, protection));
        }
        Ok(())
    }

    pub(super) fn take_original_read(
        &mut self,
    ) -> Option<crate::durability::data::OriginalCheckpointRead> {
        self.journal.take()
    }

    /// No live-head reopening: each original cell demands its own verified frame.
    pub(super) fn read_canonical(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        limit: u64,
    ) -> Result<ExecutionArray<OriginalCanonicalCommit>, Denial> {
        let covered = self
            .base
            .as_ref()
            .map_or(0, |base| base.directory.manifest.commits.len());
        let count = self
            .artifacts
            .len()
            .checked_sub(covered)
            .ok_or(Denial::OriginalAuthorityUnavailable)?;
        let mut commits =
            ExecutionRequestArrayBuilder::allocate(*execution, count).map_err(array_stop)?;
        for (offset, artifact) in self.artifacts.iter().enumerate().skip(covered) {
            contracts::execution::request_custody::check_request(execution)
                .map_err(Denial::Request)?;
            let frame = crate::durability::canonical_commit_artifact::retained_frame_reads::require_original_artifact(
                    &artifact.envelope_residency(),
                    execution,
                    self.journal
                        .as_ref()
                        .map(|(source, protection)| (source, protection)),
                )
                .map_err(capture_stop)?;
            if frame.as_bytes().len() as u64 > limit {
                return Err(contracts::storage::data::storage_unavailable::RelationalStorageUnavailable {
                    subject: contracts::storage::data::storage_unavailable::RelationalStorageSubject::Envelope(
                        PatchStreamPosition(offset as u64 + 1),
                    ),
                    cause: contracts::storage::data::storage_unavailable::RelationalStorageUnavailableCause::LimitExceeded {
                        length: frame.as_bytes().len() as u64,
                        limit,
                    },
                }
                .into());
            }
            let readmitted = frame
                .readmitted_body(execution)
                .map_err(canonical_readmission_stop)?;
            let positioned = readmitted;
            let envelope = positioned.canonical_arc().clone();
            if positioned.position().0 != offset as u64 + 1
                || envelope.commit.commit_id != artifact.commit_id()
                || !match artifact.resident_envelope() {
                    // The original v5 frame intentionally omits live diagnostics
                    // and derived caches. Compare its full positioned authority,
                    // including the sealed semantic result, against the SAME
                    // retained original envelope; never regenerate frame bytes.
                    Some(original) => crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifact::matches_positioned_payload_for_request(
                        &positioned, &original, execution,
                    ).map_err(capture_stop)?,
                    None => {
                        let (entry, _) = artifact.envelope_residency().source_placement()
                            .ok_or(Denial::OriginalAuthorityUnavailable)?;
                        let digest = worth_relational_branch::internal_port::history::commit::payload_commitment::digest_envelope_for_request(
                            entry.position, &envelope, execution,
                        ).map_err(capture_stop)?;
                        entry.commit == envelope.commit
                            && entry.schema_version == envelope.schema_version
                            && entry.payload_digest == digest
                    },
                }
            {
                return Err(Denial::OriginalAuthorityUnavailable);
            }
            commits
                .push(OriginalCanonicalCommit {
                    original_payload_digest: *frame.payload_digest().as_bytes(),
                    original_frame: frame.into_bytes(),
                    position: PatchStreamPosition(offset as u64 + 1),
                    artifact: artifact.clone(),
                    envelope,
                    inputs: artifact
                        .original_inputs()
                        .ok_or(Denial::OriginalAuthorityUnavailable)?
                        .clone(),
                })
                .map_err(array_stop)?;
        }
        commits.seal().map_err(array_stop)
    }
}

pub(super) fn canonical_readmission_stop(
    cause: contracts::durability::canonical_commit_artifact::readmission_refusal::RelationalCanonicalCommitArtifactRefusal,
) -> Denial {
    use contracts::durability::canonical_commit_artifact::readmission_refusal::RelationalCanonicalCommitArtifactRefusal as Refusal;
    match cause {
        Refusal::Request(cause) => Denial::Request(cause),
        Refusal::FieldText(cause) => {
            capture_stop(contracts::symbols::data::field_text::field_text_recovery_stop(cause))
        }
        Refusal::Content(cause) => cause.into(),
        cause @ (Refusal::Frame(_)
        | Refusal::PayloadDecode { .. }
        | Refusal::PositionDisagreement { .. }
        | Refusal::CommitIdDisagreement { .. }
        | Refusal::PositionAdmission(_)
        | Refusal::NonCanonicalPayload) => {
            Denial::Durability(contracts::durability::data::recovery_errors::DurabilityError::new(
                contracts::durability::data::recovery_errors::RecoveryFailureClass::CorruptCanonicalArtifact,
                format!("original complete canonical frame refused: {cause:?}"),
            ))
        }
        _ => unreachable!("unhandled Relational contract variant"),
    }
}

use contracts::durability::data::recovery_stop::map_stop as read_map_stop;
fn map_stop(cause: worth_execution::ExecutionOrderedMapDenial) -> Denial {
    capture_stop(read_map_stop(cause))
}

// Readmission can stop inside the schema payload, below its outer framing.
// Request and allocation refusals must remain request outcomes at capture.
fn initial_inputs_stop(
    cause: crate::durability::source_initial_inputs::RelationalInitialInputsRefusal,
) -> Denial {
    use crate::durability::source_initial_inputs::RelationalInitialInputsRefusal as Refusal;
    use contracts::durability::canonical_commit_artifact::readmission_refusal::RelationalCanonicalCommitArtifactRefusal as Payload;
    match cause {
        Refusal::Request(cause) | Refusal::Payload(Payload::Request(cause)) => {
            Denial::Request(cause)
        }
        Refusal::Native(cause) => capture_stop(cause),
        Refusal::Payload(Payload::FieldText(cause)) => {
            capture_stop(contracts::symbols::data::field_text::field_text_recovery_stop(cause))
        }
        Refusal::Payload(Payload::Content(cause)) => cause.into(),
        Refusal::Malformed
        | Refusal::Missing
        | Refusal::UnsupportedVersion(_)
        | Refusal::Source(_)
        | Refusal::Payload(_) => Denial::OriginalAuthorityUnavailable,
    }
}

#[cfg(test)]
pub mod tests;
