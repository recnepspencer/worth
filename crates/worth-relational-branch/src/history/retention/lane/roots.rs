//! The owner law admits one or two actual roots, without heap rosters.
use super::*;
#[derive(Debug, Default)]
pub(super) struct RetainedRoots {
    roots: [Option<Arc<RelationalBranchRoot>>; 2],
    len: usize,
}
impl RetainedRoots {
    pub(super) fn from_vec(roots: Vec<Arc<RelationalBranchRoot>>) -> Option<Self> {
        if roots.is_empty() || roots.len() > 2 {
            return None;
        }
        let mut result = Self::default();
        for root in roots {
            result.push(root);
        }
        Some(result)
    }
    pub(super) fn one(root: Arc<RelationalBranchRoot>) -> Self {
        Self {
            roots: [Some(root), None],
            len: 1,
        }
    }
    pub(super) fn len(&self) -> usize {
        self.len
    }
    #[cfg(test)]
    pub(super) fn is_empty(&self) -> bool {
        self.len == 0
    }
    pub(super) fn iter(&self) -> impl ExactSizeIterator<Item = &Arc<RelationalBranchRoot>> {
        self.roots[..self.len]
            .iter()
            .map(|root| root.as_ref().expect("populated fixed root slot"))
    }
    pub(super) fn clear(&mut self) {
        self.roots = [None, None];
        self.len = 0;
    }
    pub(super) fn push(&mut self, root: Arc<RelationalBranchRoot>) {
        assert!(
            self.len < 2,
            "owner root bound was checked before acquisition"
        );
        self.roots[self.len] = Some(root);
        self.len += 1;
    }
}
impl std::ops::Index<usize> for RetainedRoots {
    type Output = Arc<RelationalBranchRoot>;
    fn index(&self, index: usize) -> &Self::Output {
        self.roots[index]
            .as_ref()
            .expect("selected original root is populated")
    }
}
#[derive(Debug, Clone, Copy, Default)]
pub(super) struct RootIds {
    ids: [u64; 2],
    len: usize,
}
impl RootIds {
    pub(super) fn from_roots(roots: &RetainedRoots) -> Self {
        let mut result = Self::default();
        for root in roots.iter() {
            result.ids[result.len] = root.id();
            result.len += 1;
        }
        result
    }
    pub(super) fn one(root: u64) -> Self {
        Self {
            ids: [root, 0],
            len: 1,
        }
    }
    pub(super) fn two(first: u64, second: u64) -> Self {
        Self {
            ids: [first, second],
            len: 2,
        }
    }
    pub(super) fn len(&self) -> usize {
        self.len
    }
    pub(super) fn as_slice(&self) -> &[u64] {
        &self.ids[..self.len]
    }
    pub(super) fn iter(&self) -> std::slice::Iter<'_, u64> {
        self.as_slice().iter()
    }
}
impl std::ops::Index<usize> for RootIds {
    type Output = u64;
    fn index(&self, index: usize) -> &u64 {
        &self.as_slice()[index]
    }
}
