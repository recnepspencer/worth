//! Undo, redo and correction history refusals.
use worth_query_execution::facade::{
    correction_material::QueryCorrectionMaterialDenial as Material,
    primary_graph::{
        WorthQueryAftermathCausalityReadDenial as Lineage, WorthQueryCorrectionDerivationDenial,
        WorthQueryCorrectionSelectionDenial, WorthQueryProviderSessionControlStopKind as Stop,
    },
};
use worth_query_publication::facade::application_entry::{
    WorthQueryApplicationCorrectionDenial, WorthQueryApplicationCorrectionHistoryDenial,
    WorthQueryCorrectionHistoryDenial,
};

use crate::application_home::WorthQueryHomeUndoDenial;

use super::world_history::{ancestry, exact_history};

use super::request::{
    ALREADY_DONE, BUSY, CANCELLED, CAPACITY, INTERNAL, PERMISSION, RULE_REFUSED, STALE, STORAGE,
    TIMED_OUT, VERSION_MISMATCH,
};
use super::summary::{
    WorthQueryRefusalCategory as Category, WorthQueryRefusalSummary as Summary,
    WorthQueryUserFacingRefusal,
};

const CHANGED_SINCE: Summary = Summary::with_message(
    Category::Conflict,
    "This can't be undone because it was changed again since. Refresh to see the latest version.",
);
pub(super) const HISTORY_GONE: Summary = Summary::with_message(
    Category::NotFound,
    "The history needed to undo this is no longer kept.",
);
const NOTHING_TO_REDO: Summary = Summary::with_message(
    Category::NotFound,
    "There's nothing to redo for this change.",
);
const NOT_UNDOABLE: Summary =
    Summary::with_message(Category::Validation, "This kind of change can't be undone.");
pub(super) const OTHER_VERSION: Summary = Summary::with_message(
    Category::NotFound,
    "This change isn't part of the version you're looking at.",
);

impl WorthQueryUserFacingRefusal for WorthQueryHomeUndoDenial {
    fn user_summary(&self) -> Summary {
        match self {
            Self::Conflict => CHANGED_SINCE,
            Self::NoAlternativeAvailable => NOTHING_TO_REDO,
            Self::AutoIdempotencyKeysExhausted => Summary::with_message(
                Category::Conflict,
                "This undo was retried too many times. Refresh and try again.",
            ),
            Self::Correction(cause) => cause.user_summary(),
        }
    }
}

impl WorthQueryUserFacingRefusal for WorthQueryApplicationCorrectionDenial {
    fn user_summary(&self) -> Summary {
        match self {
            Self::Mutation(cause) => cause.user_summary(),
            Self::ExecutionRequest(cause) => cause.user_summary(),
            Self::Source(cause) => cause.user_summary(),
            Self::Derivation(cause) => cause.user_summary(),
            Self::SourceBindingUnavailable | Self::ProgramOwner(_) => VERSION_MISMATCH,
            Self::RecordedOutcomeUnavailable => HISTORY_GONE,
            Self::RequiredOutputs(_) | Self::ProductSelection(_) => BUSY,
            Self::SourceOperation(_) => INTERNAL,
        }
    }
}

impl WorthQueryUserFacingRefusal for WorthQueryCorrectionSelectionDenial {
    fn user_summary(&self) -> Summary {
        match self {
            Self::Material(cause) => material(cause),
            Self::Disclosure(cause) => cause.user_summary(),
            Self::SourceNotCorrectable => NOT_UNDOABLE,
            Self::Ancestry(cause) => ancestry(cause),
            Self::ExactHistory(cause) => exact_history(cause),
            Self::HistoryLimit
            | Self::SourceUnavailable
            | Self::SourceMemberUnavailable
            | Self::SourcePublicationMismatch => HISTORY_GONE,
            Self::StaleIncarnation | Self::HistorySource(_) => STORAGE,
            Self::ForeignPhase | Self::ForeignRuntime => INTERNAL,
        }
    }
}

impl WorthQueryUserFacingRefusal for WorthQueryCorrectionDerivationDenial {
    fn user_summary(&self) -> Summary {
        match self {
            Self::SourceDiverged | Self::OriginalSubjectChanged => CHANGED_SINCE,
            Self::SourceAlreadyReversed => Summary::with_message(
                Category::Conflict,
                "This change was already undone. Refresh to see the latest version.",
            ),
            Self::ActiveReversalConsumed => Summary::with_message(
                Category::Conflict,
                "This change was already redone. Refresh to see the latest version.",
            ),
            Self::NoActiveReversal | Self::ReapplicationUnavailable => NOTHING_TO_REDO,
            Self::Selection(cause) => cause.user_summary(),
            Self::Material(cause) => material(cause),
            Self::SubjectNotDisclosed | Self::Principal(_) => PERMISSION,
            Self::Admission(cause) => cause.user_summary(),
            Self::Scope(cause) => cause.user_summary(),
            Self::Read(cause) => cause.user_summary(),
            Self::Ancestry(cause) => ancestry(cause),
            Self::UnsupportedSourceMeaning => NOT_UNDOABLE,
            Self::MissingPreImage
            | Self::SemanticMaterialUnavailable
            | Self::SourceOutput(_)
            | Self::RestoredSupport(_)
            | Self::MissingTargetParent => HISTORY_GONE,
            Self::History(cause) => lineage(cause),
            Self::UnsupportedTargetProgram
            | Self::ProgramComparison(_)
            | Self::ProgramMeaningChanged(_)
            | Self::TargetProgram(_) => VERSION_MISMATCH,
            Self::Idempotency(_) => ALREADY_DONE,
            Self::Basis(_) | Self::Workflow(_) | Self::TargetIncarnationChanged => STALE,
            Self::Domain(_) | Self::Action(_) => RULE_REFUSED,
            Self::RequiredOutputReconciliation
            | Self::DependencyComparison(_)
            | Self::Product(_) => BUSY,
            Self::DurableStateAdmission(_) => STORAGE,
            Self::ForeignPhase
            | Self::ForeignSelection
            | Self::ForeignAlternative
            | Self::AlternativeResolutionIncomplete
            | Self::OriginalInputDecode(_)
            | Self::InputCapture(_)
            | Self::CommandKey(_) => INTERNAL,
        }
    }
}

/// The retained correction material owner's reason, in end-user terms.
fn material(cause: &Material) -> Summary {
    match cause {
        Material::RetainedMaterialExpired
        | Material::Unavailable
        | Material::MemberUnavailable
        | Material::MaterialContentUnavailable { .. }
        | Material::NativeSourceUnavailable { .. }
        | Material::OriginalBindingUnavailable { .. }
        | Material::ProgramUnavailable { .. }
        | Material::PreparationDischarged { .. } => HISTORY_GONE,
        Material::PublicationPending
        | Material::PublicationUnresolved { .. }
        | Material::PublicationNoEffect { .. }
        | Material::MaterialNotReady
        | Material::MaterialPinned { .. }
        | Material::PinAlreadyClaimed { .. } => BUSY,
        Material::TargetChanged { .. } => STALE,
        Material::UnsupportedVersion(_) => VERSION_MISMATCH,
        Material::MalformedFrame
        | Material::Checksum { .. }
        | Material::PrefixMismatch
        | Material::Storage { .. } => STORAGE,
        Material::Allocation { .. } | Material::Execution { .. } | Material::IdentityExhausted => {
            CAPACITY
        }
        Material::ForeignHome
        | Material::StaleIncarnation
        | Material::OwnerUnavailable
        | Material::AdmissionRequired
        | Material::AdmissionAlreadyRecorded
        | Material::NonPortableValue { .. }
        | Material::RetentionClassMismatch
        | Material::OriginalMismatch
        | Material::NativeRecovery { .. }
        | Material::MaterialAlreadyCaptured { .. }
        | Material::UnknownPin { .. }
        | Material::PinOwnerMismatch
        | Material::PinIntentDrift { .. } => INTERNAL,
    }
}

/// The canonical correction lineage index's own reason it could not be read.
fn lineage(cause: &Lineage) -> Summary {
    match cause {
        Lineage::ExecutionDenied(Err(Stop::Cancelled)) => CANCELLED,
        Lineage::ExecutionDenied(Err(Stop::TimedOut)) => TIMED_OUT,
        Lineage::ExecutionDenied(Ok(_))
        | Lineage::ActiveSnapshotCapacityExhausted { .. }
        | Lineage::RetentionCapacityExhausted
        | Lineage::RetentionIdentityExhausted
        | Lineage::SnapshotIdentityExhausted => CAPACITY,
        Lineage::Unavailable => HISTORY_GONE,
        Lineage::IndexUnavailable { .. } => BUSY,
        Lineage::InconsistentCanonicalRelationships => STORAGE,
        Lineage::ForeignContinuation => INTERNAL,
    }
}

impl WorthQueryUserFacingRefusal for WorthQueryCorrectionHistoryDenial {
    fn user_summary(&self) -> Summary {
        match self {
            Self::DurableHomeRequired => Summary::with_message(
                Category::Unsupported,
                "Undo history is only kept for data saved on this device.",
            ),
            Self::Material(cause) => material(cause),
            Self::Selection(cause) => cause.user_summary(),
            Self::ExactHistory(cause) => exact_history(cause),
            Self::Lineage(cause) => lineage(cause),
            Self::LineageTooLong => Summary::with_message(
                Category::Capacity,
                "This change was undone and redone too many times to list.",
            ),
            Self::ForeignPhase | Self::ReversalOutsideHistory => INTERNAL,
        }
    }
}

impl WorthQueryUserFacingRefusal for WorthQueryApplicationCorrectionHistoryDenial {
    fn user_summary(&self) -> Summary {
        match self {
            Self::ExecutionRequest(cause) => cause.user_summary(),
            Self::ProductSelection(_) => BUSY,
            Self::History(cause) => cause.user_summary(),
        }
    }
}
