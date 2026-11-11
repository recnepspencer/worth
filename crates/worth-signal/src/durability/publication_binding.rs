//! Only original medium completions can authorize publication cutover.
use super::readmission::SignalDefinitionSourceRecord;
use super::{SignalDefinitionTransitionArtifact, SignalPublicationCompletion};
use std::num::NonZeroU64;
use std::sync::Arc;
use worth_foundational::facade::{DurablePublicationFate, SignalDurableJournal};
use worth_proof::{AuthorityMarker, DurablyCommitted};

pub(crate) struct SignalCommittedAdvanceRecord {
    artifact: super::SignalBranchAdvanceArtifact,
    position: NonZeroU64,
    _definition: Arc<SignalDefinitionSourceRecord>,
}
impl SignalCommittedAdvanceRecord {
    pub(crate) fn artifact(&self) -> &super::SignalBranchAdvanceArtifact {
        &self.artifact
    }
    pub(crate) fn source_point(&self) -> super::SignalRecordedSourcePoint {
        super::SignalRecordedSourcePoint::original(self.artifact(), self.position)
    }
    pub(crate) fn position(&self) -> NonZeroU64 {
        self.position
    }
}
pub(crate) fn admit_branch_record(
    artifact: super::SignalBranchAdvanceArtifact,
    definition: Arc<SignalDefinitionSourceRecord>,
    completion: SignalPublicationCompletion,
) -> Result<
    SignalCommittedAdvanceRecord,
    (
        super::SignalBranchAdvanceArtifact,
        DurablePublicationFate<SignalPublicationBindingDenial>,
    ),
> {
    match admit_original_location(
        SignalDurableJournal::Advances,
        artifact.digest(),
        completion,
    ) {
        Ok(position) => {
            if !definition.matches(artifact.descriptor()) {
                let mismatch = SignalPublicationBindingDenial::DefinitionSourceMismatch {
                    expected: artifact.descriptor().definition(),
                    expected_branch: artifact.descriptor().branch(),
                    observed: definition.reference(),
                    observed_branch: definition.branch(),
                };
                return Err((
                    artifact,
                    DurablePublicationFate::Indeterminate { detail: mismatch },
                ));
            }
            Ok(SignalCommittedAdvanceRecord {
                artifact,
                position,
                _definition: definition,
            })
        }
        Err(fate) => Err((artifact, fate)),
    }
}

#[derive(Debug)]
pub(crate) enum SignalPublicationBindingDenial {
    DefinitionSourceMismatch {
        expected: worth_foundational::facade::SignalDefinitionTransitionReference,
        expected_branch: worth_foundational::facade::DurableSignalBranchIdentity,
        observed: worth_foundational::facade::SignalDefinitionTransitionReference,
        observed_branch: worth_foundational::facade::DurableSignalBranchIdentity,
    },
    Medium(super::SignalDurableMediumDenial),
    WrongCommittedSubject {
        expected_journal: SignalDurableJournal,
        expected_digest: [u8; 32],
        observed: Option<(SignalDurableJournal, u64, [u8; 32])>,
    },
}
/// An actual owner artifact and original physical location. Its constructor is
/// private to this medium binding; descriptions never construct this subject.
pub(crate) struct SignalCommittedDefinition {
    artifact: SignalDefinitionTransitionArtifact,
    position: NonZeroU64,
}
impl AuthorityMarker for SignalCommittedDefinition {}
impl SignalCommittedDefinition {
    pub(crate) fn into_original(self) -> (SignalDefinitionTransitionArtifact, NonZeroU64) {
        (self.artifact, self.position)
    }
}
pub(crate) fn admit_definition(
    artifact: SignalDefinitionTransitionArtifact,
    completion: SignalPublicationCompletion,
) -> Result<
    DurablyCommitted<SignalCommittedDefinition>,
    DurablePublicationFate<SignalPublicationBindingDenial>,
> {
    let position = admit_original_location(
        SignalDurableJournal::Definitions,
        artifact.digest(),
        completion,
    )?;
    Ok(DurablyCommitted::from_owner(SignalCommittedDefinition {
        artifact,
        position,
    }))
}
pub(crate) fn admit_original_location(
    journal: SignalDurableJournal,
    digest: [u8; 32],
    completion: SignalPublicationCompletion,
) -> Result<NonZeroU64, DurablePublicationFate<SignalPublicationBindingDenial>> {
    let (fate, location) = completion.into_original_parts();
    match fate {
        DurablePublicationFate::Committed => {
            if let Some((observed_journal, position, observed_digest)) = location
                && observed_journal == journal
                && observed_digest == digest
                && let Some(position) = NonZeroU64::new(position)
            {
                return Ok(position);
            }
            Err(DurablePublicationFate::Indeterminate {
                detail: SignalPublicationBindingDenial::WrongCommittedSubject {
                    expected_journal: journal,
                    expected_digest: digest,
                    observed: location,
                },
            })
        }
        DurablePublicationFate::ProvenNoEffect { detail } => {
            Err(DurablePublicationFate::ProvenNoEffect {
                detail: SignalPublicationBindingDenial::Medium(detail),
            })
        }
        DurablePublicationFate::Indeterminate { detail } => {
            Err(DurablePublicationFate::Indeterminate {
                detail: SignalPublicationBindingDenial::Medium(detail),
            })
        }
    }
}
