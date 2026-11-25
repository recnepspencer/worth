//! End-user summary of a refused Home open, classified at the Store composition.
use worth_query_declaration::facade::application_schema::ApplicationSchemaComposition;
use worth_store_application::{
    ApplicationHomeDescriptorDenial as Descriptor, ApplicationHomeOpenRefusal,
    ApplicationHomePhysicalRefusalKind,
};

use super::{
    WorthQueryApplicationHomeOpenRefused, WorthQueryApplicationHomeOpenRefusedPhase as Phase,
};
use crate::refusal_summary::{
    WorthQueryRefusalCategory as Category, WorthQueryRefusalSummary as Summary,
    WorthQueryUserFacingRefusal,
};

const CAPACITY: Summary = Summary::of(Category::Capacity);
const INTERNAL: Summary = Summary::of(Category::Internal);
const STORAGE: Summary = Summary::of(Category::Storage);
const TIMED_OUT: Summary = Summary::with_message(
    Category::Interrupted,
    "That took too long and was stopped. Nothing was changed. Try again.",
);
const VERSION_MISMATCH: Summary = Summary::with_message(
    Category::Unsupported,
    "This data needs a different version of the app. Update the app and try again.",
);
const OPEN_ELSEWHERE: Summary = Summary::with_message(
    Category::Unavailable,
    "This data is already open in another window or app. Close it there and try again.",
);
const DAMAGED: Summary = Summary::with_message(
    Category::Storage,
    "The saved data couldn't be read. It may be incomplete or damaged.",
);

impl<Schema: ApplicationSchemaComposition, Program> WorthQueryUserFacingRefusal
    for WorthQueryApplicationHomeOpenRefused<Schema, Program>
{
    fn user_summary(&self) -> Summary {
        match self.phase() {
            Phase::Physical { cause, .. } | Phase::NativeFinalizationInput { cause, .. } => {
                physical_open(cause)
            }
            Phase::Binding { .. }
            | Phase::TransitionProgrammeHandoff { .. }
            | Phase::TransitionSignalProgramPreparation { .. }
            | Phase::TransitionComposition { .. } => VERSION_MISMATCH,
            Phase::Preparation { .. } => INTERNAL,
            Phase::SignalProgramPreparation { .. }
            | Phase::RelationalAttachment { .. }
            | Phase::NativePreparation { .. }
            | Phase::NativeInstallation { .. }
            | Phase::NativeReadmission { .. }
            | Phase::QueryPublicationReadmissionRequired { .. }
            | Phase::ExistingNativeFinalization { .. }
            | Phase::ExistingBridgeDefinition { .. }
            | Phase::ExistingWorldSourcePreparation { .. }
            | Phase::ExistingWorldReadmission { .. }
            | Phase::ExistingComposition { .. }
            | Phase::ExistingSourceAttachment { .. }
            | Phase::ExistingCheckpointMaintenance { .. }
            | Phase::ExistingWorldRoot { .. }
            | Phase::ExistingQueryAttachment { .. }
            | Phase::NativeFinalization { .. }
            | Phase::BridgeDefinition { .. }
            | Phase::WorldSourcePreparation { .. }
            | Phase::WorldReadmission { .. }
            | Phase::Composition { .. }
            | Phase::SourceAttachment { .. }
            | Phase::InitialState { .. }
            | Phase::SeedPublication { .. }
            | Phase::CheckpointMaintenance { .. }
            | Phase::WorldRoot { .. }
            | Phase::QueryAttachment { .. }
            | Phase::CustodyReadmission { .. }
            | Phase::OwnerAdmission { .. } => STORAGE,
        }
    }
}

/// The physical Store's own reason a Home could not be opened.
fn physical_open(cause: &ApplicationHomeOpenRefusal) -> Summary {
    match cause {
        ApplicationHomeOpenRefusal::Execution(_) => CAPACITY,
        ApplicationHomeOpenRefusal::Physical(refusal) => match refusal.kind() {
            ApplicationHomePhysicalRefusalKind::MutationOwnerContended
            | ApplicationHomePhysicalRefusalKind::DiscardPending => OPEN_ELSEWHERE,
            ApplicationHomePhysicalRefusalKind::UnsupportedRootSchema { .. } => VERSION_MISMATCH,
            ApplicationHomePhysicalRefusalKind::PhysicalStep => STORAGE,
        },
        ApplicationHomeOpenRefusal::Descriptor(descriptor) => descriptor_open(descriptor),
        ApplicationHomeOpenRefusal::Transition(_) => VERSION_MISMATCH,
        ApplicationHomeOpenRefusal::RuntimeConstruction(_) => INTERNAL,
        ApplicationHomeOpenRefusal::Checkpoint(_)
        | ApplicationHomeOpenRefusal::Relational(_)
        | ApplicationHomeOpenRefusal::World(_)
        | ApplicationHomeOpenRefusal::SignalSourceRequired
        | ApplicationHomeOpenRefusal::SignalMedium(_)
        | ApplicationHomeOpenRefusal::SignalImage(_)
        | ApplicationHomeOpenRefusal::SignalInitial(_)
        | ApplicationHomeOpenRefusal::SignalInitialProgress(_)
        | ApplicationHomeOpenRefusal::SignalReadmission(_)
        | ApplicationHomeOpenRefusal::SignalAttachment(_)
        | ApplicationHomeOpenRefusal::BridgeInitial(_)
        | ApplicationHomeOpenRefusal::AuthoredBridge(_)
        | ApplicationHomeOpenRefusal::BridgeSource(_)
        | ApplicationHomeOpenRefusal::Bridge(_)
        | ApplicationHomeOpenRefusal::ConditionalBridge(_) => DAMAGED,
    }
}

fn descriptor_open(descriptor: &Descriptor) -> Summary {
    match descriptor {
        Descriptor::UnsupportedVersion { .. }
        | Descriptor::UnsupportedBinding { .. }
        | Descriptor::IdentityMismatch
        | Descriptor::ApplicationBindingMissing
        | Descriptor::BuildBindingMissing
        | Descriptor::BindingMismatch
        | Descriptor::TransitionPredecessorNotRostered
        | Descriptor::TransitionPredecessorMismatch
        | Descriptor::TransitionRequired { .. } => VERSION_MISMATCH,
        Descriptor::Execution(_) | Descriptor::SizeLimit => CAPACITY,
        Descriptor::Deadline(_) => TIMED_OUT,
        Descriptor::Malformed | Descriptor::IncompleteInitialization => DAMAGED,
        Descriptor::Unavailable(_)
        | Descriptor::Read(_)
        | Descriptor::Publication(_)
        | Descriptor::IncarnationExhausted => STORAGE,
    }
}
