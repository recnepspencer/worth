//! Report the typed pre-effect publication admission cause.
use super::*;
use crate::durability::canonical_commit_artifact::encode_denial::RelationalCanonicalCommitArtifactEncodeDenial as Encoding;
use crate::durability::medium::home_write_denial::RelationalHomeProgramWriteDenial as Home;
use crate::durability::medium::publication_refusal::RelationalDurablePublicationRefusal as Durable;
use crate::execution::RelationalExecutionDenialCause;
use crate::mvcc::publication::outcome::refusals::RelationalPublicationDenial as Admission;
use crate::mvcc::publication::read_dependency_denial::RelationalPublicationReadDependencyDenial as Read;
use worth_execution::report::RetryHint;
use worth_execution::{ExecutionAllocationDenialClass, ExecutionPhysicalAllocationDenialKind};
use DenialClass::*;
use Disposition::*;

pub(super) fn admission_disposition(denial: &Admission) -> (DenialClass, Disposition) {
    match denial {
        Admission::CutoverDeadlineElapsed => (DeadlineElapsed, Final),
        Admission::StaleIndexDefinitions
        | Admission::StaleConfigurationEpoch
        | Admission::StaleInvariantGeneration { .. } => (Conflict, Reauthorize),
        Admission::IndexDefinitionPublicationContended => {
            (Unavailable, Retry(RetryHint::AfterBackoff))
        }
        Admission::OwnerUnavailable { .. } | Admission::Archived | Admission::Deleting => {
            (Unavailable, Final)
        }
        Admission::ScopedCutoverDurabilityUnsupported
        | Admission::RetainedEntitySelectionDurabilityUnsupported => {
            (InvalidInput, CorrectInput)
        }
        Admission::HomeProgramTransition(denial) => home_disposition(denial),
        Admission::CanonicalArtifactEncoding(denial) => encoding_disposition(denial),
        Admission::DurablePublicationRefused(denial) => durable_disposition(denial),
        Admission::CurrentReadDependency(denial) => match denial {
            Read::NotCurrent | Read::ConflictingSource => (Conflict, Reauthorize),
            Read::CandidateUnavailable => (Unavailable, Final),
            Read::EmptySourceUnsupported | Read::AlreadyRequired => (InvalidInput, CorrectInput),
            Read::ForeignOwner => (Authorization, Reauthorize),
        },
        Admission::ForeignRuntime { .. }
        | Admission::OwnerMismatch
        | Admission::RecoveredCodecTransitionRequired
        | Admission::HomeProgramContractsRequired
        | Admission::HomeProgramIndexDefinitionsRequired => (Authorization, Reauthorize),
    }
}

fn home_disposition(denial: &Home) -> (DenialClass, Disposition) {
    match denial {
        Home::BodyCapacityOverflow => (Exhausted, Final),
        Home::BodyEncoding => (Defect, Escalate),
        Home::BodyMismatch | Home::UnsupportedBody | Home::TransitionNotRequested => {
            (InvalidInput, CorrectInput)
        }
        Home::TransitionAlreadyBound => (Conflict, Reauthorize),
        Home::MediumMoved => (Unavailable, Final),
        Home::TransitionRequired | Home::Admission(_) => (Authorization, Reauthorize),
    }
}

fn encoding_disposition(denial: &Encoding) -> (DenialClass, Disposition) {
    match denial {
        Encoding::FrameSizeOverflow | Encoding::SizeLimit { .. } => (Exhausted, CorrectInput),
        Encoding::Request(cause) => (cause.denial_class(), cause.disposition()),
        Encoding::Allocation(denial) => match denial.classify() {
            ExecutionAllocationDenialClass::Request(cause) => {
                let cause = RelationalExecutionDenialCause::from(cause);
                (cause.denial_class(), cause.disposition())
            }
            ExecutionAllocationDenialClass::Physical(
                ExecutionPhysicalAllocationDenialKind::Allocator,
            ) => (Exhausted, Final),
            ExecutionAllocationDenialClass::Physical(_) => (Defect, Escalate),
        },
        Encoding::StorageUnavailable(_) => (Unavailable, Escalate),
        Encoding::Readmission(_)
        | Encoding::Frame(_)
        | Encoding::PayloadEncoding { .. }
        | Encoding::CountEmissionMismatch { .. } => (Defect, Escalate),
    }
}

fn durable_disposition(denial: &Durable) -> (DenialClass, Disposition) {
    match denial {
        Durable::HomeProgramTransition(cause) => home_disposition(cause),
        Durable::Encoding(cause) => encoding_disposition(cause),
        Durable::TransitionEncoding(_) => (Defect, Escalate),
        Durable::MediumMoved => (Unavailable, Final),
        Durable::ProvenNoEffect { .. } => (ExternalEffect, Escalate),
    }
}

display_error!(Admission, _error => "Publication admission was refused."; error => match error {
    Admission::HomeProgramTransition(cause) => Some(cause),
    Admission::CanonicalArtifactEncoding(cause) => Some(cause),
    Admission::CurrentReadDependency(cause) => Some(cause),
    Admission::DurablePublicationRefused(cause) => Some(cause),
    _ => None,
});
display_error!(Durable, _error => "The durable medium refused the publication."; error => match error {
    Durable::HomeProgramTransition(cause) => Some(cause),
    Durable::Encoding(cause) => Some(cause),
    Durable::TransitionEncoding(cause) => Some(cause),
    Durable::MediumMoved | Durable::ProvenNoEffect { .. } => None,
});
display_error!(Home, _error => "The Home program refused the write."; _error => None);
display_error!(Read, _error => "The required publication read was refused."; _error => None);
