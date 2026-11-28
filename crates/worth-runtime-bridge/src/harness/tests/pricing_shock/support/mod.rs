pub(in crate::harness::tests::pricing_shock) use super::super::pricing_domain::{
    MaterialPriceAttribution, MaterialTickWave, PricingCommitAttribution, PricingDomainWorld,
    PricingMaterial, ProductPriceBreakdown,
};
pub(in crate::harness::tests::pricing_shock) use super::super::pricing_support::*;
pub(in crate::harness::tests::pricing_shock) use crate::adapter::TruthWritebackAuthority;
pub(in crate::harness::tests::pricing_shock) use crate::error::BridgeDeliveryErrorKind;
pub(in crate::harness::tests::pricing_shock) use crate::facade::{
    BridgeAspectRegistration, BridgeAspectRegistrationId, BridgeCommittedPatchEnvelope,
    BridgeCommittedPatchItem, BridgeDiagnosticsTier, BridgeExecutionPolicyClass,
    BridgeFailureClass, BridgeMappingId, BridgeMappingRegistration, BridgePolicyDeclaration,
    BridgePolicyDeclarationIdentity, BridgePreviewResidueClass, BridgePreviewSessionDeclaration,
    BridgePreviewSessionDeclarationIdentity, BridgeReplayErrorKind, BridgeRequestKind,
    BridgeRuntimePolicy, BridgeSignalBranchIdentity, BridgeSourceCapability,
    BridgeSourceCapabilitySet, BridgeSpeculativeBranchBinding,
    BridgeSpeculativeBranchBindingIdentity, BridgeSpeculativeSessionRequest,
    BridgeStandardRouteError, BridgeSubscriptionReferenceWorkloadComponentIdSet,
    BridgeSubscriptionReferenceWorkloadFamilyKind, BridgeSubscriptionReferenceWorkloadLaneIdSet,
    BridgeSubscriptionReferenceWorkloadLaneKind, BridgeSubscriptionReferenceWorkloadLaneRequest,
    BridgeSubscriptionReferenceWorkloadManifestSealed,
    BridgeSubscriptionReferenceWorkloadProductIdSet,
    BridgeSubscriptionReferenceWorkloadRequiredCoverageFacet,
    BridgeSubscriptionReferenceWorkloadSufficiency, BridgeTruthViewEvaluationRequest,
    BridgeTruthViewSelector, BridgeWritebackCausalityIdentity, BridgeWritebackDeclaration,
    BridgeWritebackDeclarationIdentity, BridgeWritebackEffectClass, BridgeWritebackEffectIdentity,
    BridgeWritebackEffectIntent, BridgeWritebackFamilyKind, BridgeWritebackIdempotenceClass,
    BridgeWritebackIdempotenceIdentity, BridgeWritebackRequestMode, BridgeWritebackStrategyClass,
    CoarseRoutingMode, FineGrainedMatchStatus, MappingSelector, RuntimeBridge,
    RuntimeBridgeBuilder, SignalInvalidationScope, SliceWideningPolicy, SnapshotReadRecord,
    SubscriptionSliceKind, TruthBranchIdentity, TruthCommitIdentity, TruthDeltaSurfaceKind,
    TruthPatchIdentity, TruthPatchScope, TruthSnapshotIdentity,
};
pub(in crate::harness::tests::pricing_shock) use crate::harness::fixtures::{
    BridgeHarnessFixture, InMemoryRelationalBridgeSource, RecordingSignalBridgeSink,
    RecordingTruthWritebackAuthority, SnapshotFixture,
};
pub(in crate::harness::tests::pricing_shock) use crate::snapshot::{
    SnapshotReadPacket, SnapshotReadRequest,
};
pub(in crate::harness::tests::pricing_shock) use crate::source::{
    SourceDeclaration, SourceDeclarationIdentity,
};
pub(in crate::harness::tests::pricing_shock) use crate::speculation::BridgePreviewLifecycleStateKind;
pub(in crate::harness::tests::pricing_shock) use crate::speculation::BridgePreviewSessionIdentity;
pub(in crate::harness::tests::pricing_shock) use std::collections::BTreeMap;
pub(in crate::harness::tests::pricing_shock) use worth_foundational::facade::{
    AspectKey, AspectValue,
};
pub(in crate::harness::tests::pricing_shock) use worth_harness::facade::{
    ExecutionProfile, FeedStreamEventKind, FeedVolatilityRegime, ScenarioPlan,
};

mod capture;
mod historical_portfolio;
mod pricing_snapshot_records;
mod provenance_records;
mod route_and_runtime;
mod simulation;
mod source_fixtures;
mod writeback_authority;
mod writeback_capture;

pub(in crate::harness::tests::pricing_shock) use capture::*;
pub(in crate::harness::tests::pricing_shock) use historical_portfolio::*;
pub(in crate::harness::tests::pricing_shock) use pricing_snapshot_records::*;
pub(in crate::harness::tests::pricing_shock) use provenance_records::*;
pub(in crate::harness::tests::pricing_shock) use route_and_runtime::*;
pub(in crate::harness::tests::pricing_shock) use simulation::*;
pub(in crate::harness::tests::pricing_shock) use source_fixtures::*;
pub(in crate::harness::tests::pricing_shock) use writeback_authority::*;
pub(in crate::harness::tests::pricing_shock) use writeback_capture::*;
