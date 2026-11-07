//! Signal-owned durable source, distinct from evaluation snapshots.
pub(crate) mod authored_image;
mod branch_advance;
mod definition_transition;
mod descriptor;
mod frame;
mod medium;
mod owner;
pub(crate) mod publication_binding;
pub use owner::{SignalDurableOwner, SignalReadmittedConditionalContract};
pub(crate) mod readmission;
pub use readmission::{SignalOwnerReadmissionDenial, SignalRecordedIdentityContribution};
mod tier_vocabulary;
pub use authored_image::{
    SignalAuthoredImage, SignalAuthoredImageDenial, SignalRuntimeAuthoredImage,
    SIGNAL_AUTHORED_IMAGE_PROTOCOL, SIGNAL_AUTHORED_IMAGE_VERSION,
    SIGNAL_RUNTIME_AUTHORED_IMAGE_PROTOCOL,
};
pub use branch_advance::{SignalAuthoredBranchChange, SignalBranchAdvanceArtifact};
pub use definition_transition::{SignalBridgeAuthoredSection, SignalDefinitionTransitionArtifact};
pub use descriptor::SignalDurableBranchDescriptor;
pub use frame::{
    SignalDurableFrameDenial, SIGNAL_OWNER_FRAME_MAXIMUM_BYTES, SIGNAL_OWNER_FRAME_PROTOCOL,
    SIGNAL_OWNER_FRAME_VERSION,
};
pub use medium::{
    SignalDurableHomeBinding, SignalDurableMedium, SignalDurableMediumDenial,
    SignalDurableReadSource, SignalPublicationCompletion, SignalPublicationObservation,
    SignalPublicationTaskPosture,
};
pub use tier_vocabulary::{SignalDurableTierVocabulary, SignalDurableTierVocabularyDenial};
