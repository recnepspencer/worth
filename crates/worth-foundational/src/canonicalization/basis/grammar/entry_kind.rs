use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum CanonicalBasisEntryKind {
    Header,
    Shape,
    Value,
    Field,
    Mask,
    StateAspect,
    PatchOperation,
    Identity,
    Locator,
    Profile,
    PerformanceClaim,
    PerformanceLayout,
    PerformanceCounter,
    PerformanceSupport,
    BoundaryArtifact,
    BoundaryAttachment,
    TransitionArtifact,
    TransitionLocator,
    DiagnosticBundle,
    DiagnosticRow,
    DiagnosticGap,
    CompatibilityOrigin,
    Cost,
    #[serde(skip)]
    Future(#[serde(skip)] &'static str),
}
