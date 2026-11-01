//! Read constraints and failure observations carry no graph or success authority.
use super::*;

pub struct PinnedDerivedRelationScope<'input> {
    pub(super) definition: &'input Handle,
    pub(super) root: NativeMemberKey,
    pub(super) shape: super::super::DerivedRelationReadShape,
    pub(super) maximum_work: Option<usize>,
    pub(super) control: Option<&'input dyn Fn() -> Result<(), Cause>>,
}
impl<'input> PinnedDerivedRelationScope<'input> {
    pub fn new(
        definition: &'input Handle,
        root: impl Into<NativeMemberKey>,
        shape: super::super::DerivedRelationReadShape,
    ) -> Self {
        Self {
            definition,
            root: root.into(),
            shape,
            maximum_work: None,
            control: None,
        }
    }
    pub fn with_work_limit(mut self, maximum: Option<usize>) -> Self {
        self.maximum_work = maximum;
        self
    }
    /// This checkpoint can only narrow the original request. A richer caller
    /// can retain its first stop externally and return NestedStopped here.
    pub fn with_control(mut self, control: &'input dyn Fn() -> Result<(), Cause>) -> Self {
        self.control = Some(control);
        self
    }
}

/// Actual observations survive refusal; partial endpoint data never escapes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PinnedDerivedRelationDenial {
    pub kind: Denial,
    pub reads: Reads,
    pub work: usize,
}
impl std::fmt::Display for PinnedDerivedRelationDenial {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "pinned derived source refused: {:?}", self.kind)
    }
}
impl std::error::Error for PinnedDerivedRelationDenial {}
