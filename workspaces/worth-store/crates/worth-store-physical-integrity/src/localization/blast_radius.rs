#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalBlastRadius {
    DamagedRange,
    CanonicalFrame,
    CompleteArtifact,
    ReachableSubtree,
}
