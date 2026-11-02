//! Framework-only AVL links, ordered extrema and qualified summary repair.
use super::*;
impl<P, R, S: Summary> PredicateTree<P, R, S> {
    pub(super) fn repair(&mut self, id: WatchSlot) {
        let node = self.node(id);
        let [left, right] = node.children;
        let height = self.height(left).max(self.height(right)) + 1;
        let extrema = [
            left.map_or(id, |l| self.node(l).extrema[0]),
            right.map_or(id, |r| self.node(r).extrema[1]),
        ];
        let aggregate = S::merge(
            node.own,
            [
                left.map(|l| self.node(l).aggregate),
                right.map(|r| self.node(r).aggregate),
            ],
        );
        let node = self.node_mut(id);
        node.height = height;
        node.extrema = extrema;
        node.aggregate = aggregate;
    }

    pub(super) fn replace_link(&mut self, old: WatchSlot, new: Option<WatchSlot>) {
        let parent = self.node(old).parent;
        if let Some(parent) = parent {
            let side = usize::from(self.node(parent).children[1] == Some(old));
            self.node_mut(parent).children[side] = new;
        } else {
            self.root = new;
        }
        if let Some(new) = new {
            self.node_mut(new).parent = parent;
        }
    }

    fn rotate(&mut self, old: WatchSlot, side: usize) -> WatchSlot {
        let raised = self.node(old).children[side].expect("rotation's taller child exists");
        let middle = self.node(raised).children[1 - side];
        self.replace_link(old, Some(raised));
        self.node_mut(old).children[side] = middle;
        if let Some(middle) = middle {
            self.node_mut(middle).parent = Some(old);
        }
        self.node_mut(raised).children[1 - side] = Some(old);
        self.node_mut(old).parent = Some(raised);
        self.repair(old);
        self.repair(raised);
        raised
    }

    pub(super) fn rebalance(&mut self, mut next: Option<WatchSlot>) {
        while let Some(id) = next {
            self.repair(id);
            let [left, right] = self.node(id).children;
            let balance = i16::from(self.height(left)) - i16::from(self.height(right));
            let root = if balance.abs() > 1 {
                let side = usize::from(balance < 0);
                let child = self.node(id).children[side].expect("imbalanced side has a child");
                let grandchildren = self.node(child).children;
                if self.height(grandchildren[1 - side]) > self.height(grandchildren[side]) {
                    self.rotate(child, 1 - side);
                }
                self.rotate(id, side)
            } else {
                id
            };
            next = self.node(root).parent;
        }
    }
}
