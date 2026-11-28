use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum CanonicalBasisDomain {
    Value,
    AspectContract,
    AspectMask,
    AuthoritativeState,
    AuthoritativePatch,
    Identity,
    Locator,
    Profile,
    Performance,
    BoundaryArtifact,
    Transition,
    Diagnostic,
    CompatibilityLowering,
    #[serde(skip)]
    Future(#[serde(skip)] &'static str),
}
