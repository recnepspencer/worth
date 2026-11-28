use super::*;
use crate::durability::data::recovery_errors::{DurabilityError, RecoveryFailureClass};
use crate::errors::data::RelationalError;
use crate::history::data::branch_creation::{BranchCreateError, BranchCreateErrorClass};
use crate::publication::data::publication_error::PublicationError;
use crate::replay::data::replay_errors::{ReplayError, ReplayFailureClass};
use crate::schema::data::{SchemaRegistryError, SchemaRegistryErrorClass};
use crate::validation::data::results::{InvariantViolation, InvariantViolationFields};
use worth_execution::report::RetryHint;
use DenialClass::*;
use Disposition::*;

impl RelationalError {
    fn boundary_cause(&self) -> &dyn Reportable {
        match self {
            Self::Transaction(error) => error,
            Self::Durability(error) => error,
            Self::History(error) => error,
            Self::Schema(error) => error,
            Self::Publication(error) => error,
            Self::Replay(error) => error,
        }
    }
}

impl Error for RelationalError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(self.boundary_cause())
    }
}

impl Reportable for RelationalError {
    fn denial_class(&self) -> DenialClass {
        self.boundary_cause().denial_class()
    }

    fn disposition(&self) -> Disposition {
        self.boundary_cause().disposition()
    }

    fn report_code(&self) -> ReportCode {
        self.boundary_cause().report_code()
    }

    fn diagnostic_summary(&self) -> &'static str {
        self.boundary_cause().diagnostic_summary()
    }
}

report!(DurabilityError, error => {
    use RecoveryFailureClass as Cause;
    let (class, disposition) = match &error.class {
        Cause::CheckpointPublicationInFlight | Cause::PerformedPublicationRequiresSettlement =>
            (Unavailable, Retry(RetryHint::AfterBackoff)),
        Cause::CheckpointSizeOverflow | Cause::CheckpointAllocationUnavailable => (Exhausted, Final),
        Cause::DurableIoFailure | Cause::ColdStorageUnavailable => (Unavailable, Escalate),
        Cause::DeclaredSchemaMismatch | Cause::ProfileMismatch | Cause::RuntimeNameMismatch => (InvalidInput, CorrectInput),
        Cause::UnsupportedBranchRootImageVersion { .. } | Cause::UnsupportedCheckpointVersion { .. }
        | Cause::UnsupportedLegacySemantics
        | Cause::UnsupportedRetainedEntitySelectionReplay | Cause::UnsupportedOwnedGraphSelectionReplay => (Unavailable, Final),
        Cause::SchemaMismatch | Cause::CorruptCheckpoint | Cause::CorruptCanonicalArtifact
        | Cause::NonContiguousCanonicalTail | Cause::CorruptBranchTransitionArtifact
        | Cause::NonContiguousBranchTransitionTail | Cause::MissingAuthoritativeParentClosure
        | Cause::ReplayFailure | Cause::CheckpointFrameSizeMismatch => (Defect, Escalate),
    };
    (class, disposition, "relational.durability.denied", "The durability operation was refused.")
});
display_error!(DurabilityError, error => &error.detail; error => error.storage_unavailable.as_ref().map(|cause| cause as &dyn Error));

report!(BranchCreateError, error => {
    use BranchCreateErrorClass as Cause;
    let (class, code) = match error.class {
        Cause::BranchAlreadyExists => (Conflict, "relational.branch.already_exists"),
        Cause::SourceBranchMissing => (NotFound, "relational.branch.source_missing"),
        Cause::InvalidTarget => (InvalidInput, "relational.branch.invalid_target"),
    };
    (class, CorrectInput, code, "Branch creation was refused.")
});
display_error!(BranchCreateError, error => &error.detail; _error => None);

report!(SchemaRegistryError, error => {
    use SchemaRegistryErrorClass as Cause;
    let (class, disposition) = match error.class {
        Cause::UnknownEntityKind(_) | Cause::UnknownRelationKind(_) => (NotFound, CorrectInput),
        Cause::DuplicateEntityKind(_) | Cause::DuplicateRelationKind(_)
        | Cause::EntityRelationKindCollision(_) | Cause::DuplicateAspectKey { .. }
        | Cause::DuplicateRelationContractId { .. } => (Conflict, CorrectInput),
        Cause::InconsistentSchemaBasis { .. } => (Defect, Escalate),
        Cause::InvalidAspectDeclaration { .. } | Cause::InvalidRelationIntegrityDeclaration { .. } => (InvalidInput, CorrectInput),
    };
    (class, disposition, "relational.schema.denied", "The schema registry refused the declaration or lookup.")
});
display_error!(SchemaRegistryError, error => &error.detail; _error => None);

report!(ReplayError, error => {
    use ReplayFailureClass as Cause;
    let (class, disposition) = match &error.class {
        Cause::Execution(cause) => (cause.denial_class(), cause.disposition()),
        Cause::MissingCommit => (NotFound, CorrectInput),
        Cause::MissingAuthoritativeParentClosure | Cause::AuthoritativeBasisUnavailable => (Unavailable, Escalate),
        Cause::BranchMismatch | Cause::SchemaMismatch => (InvalidInput, CorrectInput),
        Cause::ObservableMismatch => (Defect, Escalate),
    };
    (class, disposition, "relational.replay.denied", "Canonical replay did not succeed.")
});
display_error!(ReplayError, error => &error.detail; error => match &error.class {
    ReplayFailureClass::Execution(cause) => Some(cause),
    _ => None,
});

report!(PublicationError, error => {
    let (class, disposition) = if let Some(cause) = error.lineage_name_denial() {
        (cause.denial_class(), cause.disposition())
    } else if error.invariant_violation().is_some() {
        (Invariant, CorrectInput)
    } else if error.authored_retirement_refusal().is_some() {
        (Authorization, Reauthorize)
    } else if error.index_maintenance_denial().is_some() {
        (Unavailable, Escalate)
    } else {
        (Defect, Escalate)
    };
    (class, disposition, "relational.publication.denied", "Publication could not finish.")
});
display_error!(PublicationError, error => &error.detail; error => {
    if let Some(cause) = error.lineage_name_denial() {
        Some(cause)
    } else if let Some(cause) = error.invariant_violation() {
        Some(cause)
    } else if let Some(cause) = error.index_maintenance_denial() {
        Some(cause)
    } else if let Some(cause) = error.authored_retirement_source() {
        Some(cause)
    } else {
        None
    }
});

display_error!(InvariantViolation, error => &error.detail; error => match &error.fields {
    InvariantViolationFields::CustomInvariantViolation { refusal, .. } => Some(refusal),
    InvariantViolationFields::CustomInvariantFailure { cause: Some(cause), .. } => Some(cause),
    _ => None,
});

use crate::validation::data::custom_rule::failure_cause::CustomInvariantFailureCause;
display_error!(CustomInvariantFailureCause, _error => "Custom invariant evaluation failed."; error => match error {
    CustomInvariantFailureCause::DeclaredValue(refusal) => Some(refusal),
    CustomInvariantFailureCause::Content(cause) => Some(cause),
    _ => None,
});

use crate::indexes::data::maintenance::{DerivedIndexMaintenanceDenial, DerivedIndexMaintenanceDenialKind};
display_error!(DerivedIndexMaintenanceDenial,
    _error => "Derived index maintenance was refused."; error => match &error.kind {
        DerivedIndexMaintenanceDenialKind::Basis(cause) => Some(cause),
        DerivedIndexMaintenanceDenialKind::Execution(cause) => Some(cause),
        _ => None,
    });
display_error!(crate::publication::data::authored_retirement_refusal::AuthoredRetirementRefusal,
    _error => "Authored retirement was refused."; _error => None);
