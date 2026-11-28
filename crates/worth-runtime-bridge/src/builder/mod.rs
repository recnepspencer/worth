use std::sync::Arc;

use crate::adapter::{
    BridgeSourceAdapter, CommittedPatchSource, ContinuityLineageSource, InvalidationSink,
    SnapshotReadSource, SnapshotReaderPool, TruthBranchHeadSource, TruthWritebackAuthority,
};
use crate::correspondence::{
    AdmittedSemanticDependencyRegistry, BridgeSemanticCorrespondenceRegistration,
};
use crate::error::{BridgeBuildError, BridgeBuildErrorKind};
use crate::facade::RuntimeBridge;
use crate::mapping::{
    BridgeAspectRegistration, BridgeMappingRegistration, FrozenAspectMappingRegistry,
    FrozenMappingRegistry,
};
use crate::policy::BridgeRuntimePolicy;
use crate::source::{AdmittedSourceRegistry, SourceDeclaration};

mod build;
mod policy;
mod registrations;
mod sources;
mod states;

#[cfg(test)]
mod tests;

pub use states::{
    MissingCommittedPatchSource, MissingMappingRegistrations, MissingSignalSink,
    MissingSnapshotReadSource, MissingTruthBranchHeadSource, PresentCommittedPatchSource,
    PresentMappingRegistrations, PresentSignalSink, PresentSnapshotReadSource,
    PresentTruthBranchHeadSource, RuntimeBridgeBuilder,
};
/// Authored Bridge configuration awaiting its concrete Relational source.
pub type UnattachedRelationalBridgeBuilder = RuntimeBridgeBuilder<
    MissingCommittedPatchSource,
    MissingSnapshotReadSource,
    PresentSignalSink,
    MissingTruthBranchHeadSource,
    PresentMappingRegistrations,
>;
