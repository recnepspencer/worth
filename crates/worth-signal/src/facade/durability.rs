//! Authored source transport. Descriptive bytes never grant a live owner basis.
pub use crate::durability::{
    SignalAuthoredBranchChange, SignalAuthoredImage, SignalAuthoredImageDenial,
    SignalBranchAdvanceArtifact, SignalBridgeAuthoredSection, SignalDefinitionTransitionArtifact,
    SignalDurableBranchDescriptor, SignalDurableFrameDenial, SignalDurableHomeBinding,
    SignalDurableMedium, SignalDurableMediumDenial, SignalDurableReadSource,
    SignalPublicationCompletion, SignalPublicationObservation, SignalPublicationTaskPosture,
    SIGNAL_AUTHORED_IMAGE_PROTOCOL, SIGNAL_AUTHORED_IMAGE_VERSION,
    SIGNAL_OWNER_FRAME_MAXIMUM_BYTES, SIGNAL_OWNER_FRAME_PROTOCOL, SIGNAL_OWNER_FRAME_VERSION,
};
