//! The single AVL balancing decision, before any cell allocation.
use super::*;
impl<'tree, K, V> Plan<'_, 'tree, K, V> {
    pub(super) fn balanced(
        &mut self,
        entry: Option<&'tree Arc<Entry<K, V>>>,
        left: PlannedLink<'tree, K, V>,
        right: PlannedLink<'tree, K, V>,
    ) -> Result<PlannedLink<'tree, K, V>, Denial> {
        if self.height(left) > self.height(right) + 1 {
            let child = self.view(left).expect("left-heavy node");
            if self.height(child.left) >= self.height(child.right) {
                let right = self.build(entry, child.right, right)?;
                self.build(child.entry, child.left, right)
            } else {
                let pivot = self.view(child.right).expect("inner pivot");
                let left = self.build(child.entry, child.left, pivot.left)?;
                let right = self.build(entry, pivot.right, right)?;
                self.build(pivot.entry, left, right)
            }
        } else if self.height(right) > self.height(left) + 1 {
            let child = self.view(right).expect("right-heavy node");
            if self.height(child.right) >= self.height(child.left) {
                let left = self.build(entry, left, child.left)?;
                self.build(child.entry, left, child.right)
            } else {
                let pivot = self.view(child.left).expect("inner pivot");
                let left = self.build(entry, left, pivot.left)?;
                let right = self.build(child.entry, pivot.right, child.right)?;
                self.build(pivot.entry, left, right)
            }
        } else {
            self.build(entry, left, right)
        }
    }
}
