//! Signal-owned durable source, distinct from evaluation snapshots.
pub(crate) mod authored_image;
mod branch_advance;
mod committed_advance;
pub use committed_advance::SignalCommittedAdvanceEvidence;
mod composite_binding;
pub(crate) use composite_binding::fork_commitment;
mod composite_owner;
pub use composite_binding::SignalCompositeAdvanceBinding;
pub use composite_owner::SignalCompositeOwnerServicePorts;
mod definition_publication;
mod fork_publication;
pub use fork_publication::{
    SignalDurableForkPublication, SignalForkPublicationContext, SignalOnlyForkPublication,
};
mod projection_publication;
pub use projection_publication::SignalDurableProjectionPublication;
mod definition_transition;
pub(crate) use definition_publication::SignalDurableConditionalDefinitionChange;
pub use definition_publication::{
    SignalDefinitionPublicationContext, SignalDurableDefinitionPublication,
    SignalOnlyDefinitionPublication,
};
mod descriptor;
mod source_point;
pub use source_point::SignalRecordedSourcePoint;
mod readmitted_source_point;
pub use readmitted_source_point::{
    SignalReadmittedSourcePoint, SignalSourcePointReadmissionDenial,
};
mod frame;
mod medium;
mod owner;
mod owner_installation;
pub use owner_installation::{
    SignalAuthoredBranchInstallation, SignalAuthoredInstallationRefused,
    SignalAuthoredOwnerInstallation, SignalOnlyOwnerInstallation,
};
mod owner_admission;
pub(crate) use owner_admission::SignalDurableOwnerAdmission;
pub use owner_admission::{SignalDurableOwnerAdmissionRegistry, SignalLiveOwnerIdentity};
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
