//! Authored source transport. Descriptive bytes never grant a live owner basis.
pub use crate::branch::owner_services::{
    SignalDurableAdvanceCompletion, SignalDurableAdvanceDenial, SignalDurableAdvanceDriveRefusal,
    SignalDurableAdvanceOutput, SignalDurableAdvanceProgress, SignalDurableAdvanceTask,
    SignalDurableBranchMutationPort, SignalDurableTransactionResult,
};
pub use crate::branch::owner_services::{
    SignalDurableBranchHandle, SignalDurableBranchSourcePort, SignalDurableSourceDenial,
};
pub use crate::data::graph::SignalAuthoredGraphReadmission;
pub use crate::durability::{
    SignalAuthoredBranchChange, SignalAuthoredBranchInstallation, SignalAuthoredImage,
    SignalAuthoredImageDenial, SignalAuthoredInstallationRefused, SignalAuthoredOwnerInstallation,
    SignalBranchAdvanceArtifact, SignalBridgeAuthoredSection, SignalDefinitionPublicationContext,
    SignalDefinitionTransitionArtifact, SignalDurableBranchDescriptor,
    SignalDurableDefinitionPublication, SignalDurableFrameDenial, SignalDurableHomeBinding,
    SignalDurableMedium, SignalDurableMediumDenial, SignalDurableOwner,
    SignalDurableOwnerAdmissionRegistry, SignalDurableReadSource, SignalDurableTierVocabulary,
    SignalDurableTierVocabularyDenial, SignalLiveOwnerIdentity, SignalOnlyDefinitionPublication,
    SignalOnlyOwnerInstallation, SignalOwnerReadmissionDenial, SignalPublicationCompletion,
    SignalPublicationObservation, SignalPublicationTaskPosture,
    SignalReadmittedConditionalContract, SignalRecordedIdentityContribution,
    SignalRuntimeAuthoredImage, SIGNAL_AUTHORED_IMAGE_PROTOCOL, SIGNAL_AUTHORED_IMAGE_VERSION,
    SIGNAL_OWNER_FRAME_MAXIMUM_BYTES, SIGNAL_OWNER_FRAME_PROTOCOL, SIGNAL_OWNER_FRAME_VERSION,
    SIGNAL_RUNTIME_AUTHORED_IMAGE_PROTOCOL,
};
