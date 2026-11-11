//! Resident contents and their original physical source share one cell gate.
use super::ResidencyState;
use crate::durability::medium::NativeSourceMemberState;

#[derive(Debug)]
pub(super) struct RegionState {
    pub(super) contents: ResidencyState,
    pub(super) physical: Option<NativeSourceMemberState>,
}
impl RegionState {
    pub(super) fn new(contents: ResidencyState, physical: Option<NativeSourceMemberState>) -> Self {
        Self { contents, physical }
    }
}
impl std::ops::Deref for RegionState {
    type Target = ResidencyState;
    fn deref(&self) -> &Self::Target {
        &self.contents
    }
}
impl std::ops::DerefMut for RegionState {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.contents
    }
}
