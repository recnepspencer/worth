mod artifact;
mod bound;
mod dependency;
mod equivalence;
mod identity_seeded;
#[cfg(test)]
mod inspection;
mod lower;
mod resolved;
mod structured;
#[cfg(test)]
pub(crate) mod test_compilation;

pub(crate) use artifact::{
    WorthUiArtifact, WorthUiArtifactBindingHandle, WorthUiArtifactBindingNode,
    WorthUiArtifactComponentHandle, WorthUiArtifactComponentNode, WorthUiArtifactHandle,
    WorthUiArtifactImportHandle, WorthUiArtifactImportNode, WorthUiArtifactModule,
    WorthUiArtifactNode, WorthUiArtifactNodeKind, WorthUiArtifactSurfaceHandle,
    WorthUiArtifactSurfaceNode, WorthUiArtifactSurfaceNodeInput, WorthUiArtifactThemeTokenHandle,
    WorthUiArtifactThemeTokenNode,
};
#[cfg(test)]
pub(crate) use bound::WorthUiBoundArtifactInputEquivalentShape;
pub(crate) use bound::{
    WorthUiBoundArtifactInput, WorthUiBoundArtifactInputBindingNode,
    WorthUiBoundArtifactInputComponentNode, WorthUiBoundArtifactInputModule,
    WorthUiBoundArtifactInputNode, WorthUiBoundArtifactInputSurfaceNode,
    WorthUiBoundArtifactInputThemeTokenNode, WorthUiBoundCommandProjectionReference,
    WorthUiBoundCommandReference, WorthUiBoundCommandSemantics, WorthUiBoundIconReference,
    WorthUiBoundQueryViewSemantics, WorthUiBoundSurfaceSemantics, WorthUiBoundThemeTokenSemantics,
    WorthUiBoundViewBindingReference,
};
pub use dependency::WorthUiArtifactSubtreeDigest;
#[cfg(test)]
pub(crate) use dependency::WorthUiRuntimeQuerySurface;
pub(crate) use dependency::{
    WorthUiArtifactDependencyEdge, WorthUiArtifactDependencyEdgeKind,
    WorthUiArtifactDependencyGraph, WorthUiArtifactDependencyTarget, WorthUiArtifactImpactMetadata,
    WorthUiIncrementalInvalidationBasis, WorthUiRuntimeDependencyHook,
    WorthUiRuntimeDependencyHookKind,
};
pub(crate) use equivalence::WorthUiArtifactEquivalenceDenial;
pub(crate) use equivalence::{
    WorthUiArtifactDifference, WorthUiArtifactDigest, WorthUiArtifactDigestReport,
    WorthUiArtifactEquivalence, WorthUiArtifactEquivalenceBasis, WorthUiArtifactSemanticDelta,
};
#[cfg(test)]
pub(crate) use identity_seeded::WorthUiIdentityReplacementClass;
#[cfg(test)]
pub(crate) use identity_seeded::WorthUiIdentitySeededArtifactInputEquivalentShape;
pub(crate) use identity_seeded::{
    WorthUiArtifactIdentitySeed, WorthUiArtifactIdentitySeedKind, WorthUiDurableStateEligibility,
    WorthUiDurableStateIneligibilityReason, WorthUiIdentitySeededArtifactInput,
    WorthUiIdentitySeededArtifactInputBindingNode, WorthUiIdentitySeededArtifactInputComponentNode,
    WorthUiIdentitySeededArtifactInputImportNode, WorthUiIdentitySeededArtifactInputModule,
    WorthUiIdentitySeededArtifactInputNode, WorthUiIdentitySeededArtifactInputSurfaceNode,
    WorthUiIdentitySeededArtifactInputThemeTokenNode,
};
#[cfg(test)]
pub(crate) use inspection::{
    WorthUiArtifactCapabilityReference, WorthUiArtifactCapabilityReferenceInspection,
    WorthUiArtifactCapabilityReferenceRole, WorthUiArtifactInspection,
    WorthUiArtifactNodeInspection, WorthUiArtifactProvenanceMap, WorthUiArtifactSourceOrigin,
    WorthUiQueryInspectionLink, WorthUiQueryInspectionLinkRole,
};
pub(crate) use lower::{
    WorthUiArtifactAssemblyDiagnostic, WorthUiArtifactAssemblyMetrics,
    WorthUiArtifactAssemblyReport, WorthUiArtifactDependencyDeriver,
    WorthUiArtifactDependencyMetrics, WorthUiArtifactDependencyReport, WorthUiArtifactDigestor,
    WorthUiArtifactEquivalenceComparator, WorthUiArtifactEquivalenceMetrics,
    WorthUiArtifactInputResolver, WorthUiBindingDiagnostic, WorthUiBindingDiagnosticCode,
    WorthUiBindingSemanticsLowerer, WorthUiBindingSemanticsMetrics, WorthUiBindingSemanticsReport,
    WorthUiCanonicalArtifactAssembler, WorthUiIdentitySeedLowerer,
    WorthUiIdentitySeedingDiagnostic, WorthUiIdentitySeedingMetrics, WorthUiIdentitySeedingReport,
    WorthUiResolutionDiagnostic, WorthUiResolutionDiagnosticCode, WorthUiResolutionMetrics,
    WorthUiResolutionReport, WorthUiStructuralLegalityDiagnostic,
    WorthUiStructuralLegalityDiagnosticCode, WorthUiStructuralLegalityLowerer,
    WorthUiStructuralLegalityMetrics, WorthUiStructuralLegalityReport,
};
#[cfg(test)]
pub(crate) use lower::{
    WorthUiArtifactAssemblyDiagnosticCode, WorthUiArtifactInspectionBasisBuilder,
    WorthUiArtifactInspectionDeriver, WorthUiArtifactInspectionDiagnosticCode,
    WorthUiIdentityReplacementClassifier, WorthUiIdentitySeedingDiagnosticCode,
};
#[cfg(test)]
pub(crate) use lower::{
    WorthUiArtifactInspectionBasis, WorthUiArtifactInspectionDiagnostic,
    WorthUiArtifactInspectionMetrics, WorthUiArtifactInspectionReport,
};
#[cfg(test)]
pub(crate) use resolved::WorthUiResolvedArtifactInputEquivalentShape;
pub(crate) use resolved::{
    WorthUiResolvedArtifactInput, WorthUiResolvedArtifactInputBindingNode,
    WorthUiResolvedArtifactInputComponentNode, WorthUiResolvedArtifactInputModule,
    WorthUiResolvedArtifactInputNode, WorthUiResolvedArtifactInputSurfaceNode,
    WorthUiResolvedArtifactInputThemeTokenNode, WorthUiResolvedThemeTokenBindingTarget,
    WorthUiRuntimeSemanticImport,
};
#[cfg(test)]
pub(crate) use structured::WorthUiLegallyStructuredArtifactInputEquivalentShape;
pub(crate) use structured::{
    WorthUiLegallyStructuredArtifactInput, WorthUiLegallyStructuredArtifactInputBindingNode,
    WorthUiLegallyStructuredArtifactInputComponentNode,
    WorthUiLegallyStructuredArtifactInputModule, WorthUiLegallyStructuredArtifactInputNode,
    WorthUiLegallyStructuredArtifactInputSurfaceNode,
    WorthUiLegallyStructuredArtifactInputThemeTokenNode, WorthUiMosaicMountFacts,
    WorthUiMosaicRegionFacts, WorthUiMosaicSizingContractProjectionDenial,
    WorthUiMosaicStructureFacts,
};

#[cfg(test)]
mod tests;
