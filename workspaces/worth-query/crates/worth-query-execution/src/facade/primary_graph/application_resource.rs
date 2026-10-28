//! Framework-owned application resource lifecycle contracts.
pub use crate::domain_computation::primary_graph::application_resource::{
    WorthQueryApplicationResource, WorthQueryApplicationResourceProvider,
    WorthQueryApplicationResourceReceipt, WorthQueryApplicationResourcesRequest,
    WorthQueryClosedResource, WorthQueryResourceActionDenial, WorthQueryResourceCleanupDetail,
    WorthQueryResourceCleanupPosture, WorthQueryResourceEffectRefusal, WorthQueryResourceFrameKind,
    WorthQueryResourceIngressDenial, WorthQueryResourceIngressWriter,
    WorthQueryResourceOwnedEffectStartStop, WorthQueryResourceProviderAttempt,
    WorthQueryResourceProviderFailure, WorthQueryResourceRecoveryCloseStop,
    WorthQueryResourceStartStop, WorthQueryResourceState, WorthQueryResourceStopKind,
    WorthQueryResourceVerificationFailure, WorthQueryResourceVerifiedFrame,
};
