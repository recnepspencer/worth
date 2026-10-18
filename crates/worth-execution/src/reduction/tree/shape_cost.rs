//! Borrowed native shape edits use the value tree's one Cartesian traversal.
use super::{
    edit::structure::{self, Structure},
    Node, ReductionDenial, ReductionTree,
};
use crate::{ExecutionAllocationDenial, ExecutionAllocationPolicy, ExecutionMemoryReservation};
use std::sync::Arc;
use worth_foundational::PartitionIdentity;
use worth_proof::CanonicalUniqueVec;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReductionShapeCostDenial {
    Allocation(ExecutionAllocationDenial),
    Structural(ReductionDenial),
    CapacityOverflow,
}
impl From<ExecutionAllocationDenial> for ReductionShapeCostDenial {
    fn from(value: ExecutionAllocationDenial) -> Self {
        Self::Allocation(value)
    }
}
enum ShapeLink<'a, T> {
    Empty,
    Borrowed(&'a Node<T>),
    Edited(Arc<ShapeNode<'a, T>>),
}
impl<T> Clone for ShapeLink<'_, T> {
    fn clone(&self) -> Self {
        match self {
            Self::Empty => Self::Empty,
            Self::Borrowed(n) => Self::Borrowed(n),
            Self::Edited(n) => Self::Edited(Arc::clone(n)),
        }
    }
}
struct ShapeNode<'a, T> {
    id: PartitionIdentity,
    left: ShapeLink<'a, T>,
    right: ShapeLink<'a, T>,
    span: u64,
    _memory: Option<ExecutionMemoryReservation>,
}
impl<'a, T> ShapeLink<'a, T> {
    fn borrowed(node: Option<&'a Node<T>>) -> Self {
        node.map_or(Self::Empty, Self::Borrowed)
    }
    fn span(&self) -> u64 {
        match self {
            Self::Empty => 0,
            Self::Borrowed(n) => n.full_subtree_span,
            Self::Edited(n) => n.span,
        }
    }
}
struct ShapeEngine<'a, 'lease, 'request, T> {
    policy: ExecutionAllocationPolicy<'lease, 'request>,
    borrowed: std::marker::PhantomData<&'a T>,
}
impl<'a, T> Structure for ShapeEngine<'a, '_, '_, T> {
    type Link = ShapeLink<'a, T>;
    type Error = ReductionShapeCostDenial;
    fn inspect(&mut self) -> Result<(), Self::Error> {
        self.policy.check_live().map_err(Into::into)
    }
    fn parts(&self, link: &Self::Link) -> Option<(PartitionIdentity, Self::Link, Self::Link)> {
        match link {
            ShapeLink::Empty => None,
            ShapeLink::Borrowed(n) => Some((
                n.identity,
                ShapeLink::borrowed(n.left.as_deref()),
                ShapeLink::borrowed(n.right.as_deref()),
            )),
            ShapeLink::Edited(n) => Some((n.id, n.left.clone(), n.right.clone())),
        }
    }
    fn leaf(&mut self, id: PartitionIdentity) -> Result<Self::Link, Self::Error> {
        self.node(id, ShapeLink::Empty, ShapeLink::Empty)
    }
    fn build(
        &mut self,
        source: &Self::Link,
        left: Self::Link,
        right: Self::Link,
    ) -> Result<Self::Link, Self::Error> {
        let (id, _, _) = self.parts(source).expect("nonempty shape source");
        self.node(id, left, right)
    }
    fn unknown(&self, id: PartitionIdentity) -> Self::Error {
        ReductionShapeCostDenial::Structural(ReductionDenial::UnknownIdentity(id))
    }
}
impl<'a, T> ShapeEngine<'a, '_, '_, T> {
    fn node(
        &mut self,
        id: PartitionIdentity,
        left: ShapeLink<'a, T>,
        right: ShapeLink<'a, T>,
    ) -> Result<ShapeLink<'a, T>, ReductionShapeCostDenial> {
        let span = left
            .span()
            .max(right.span())
            .checked_add(4)
            .ok_or(ReductionShapeCostDenial::CapacityOverflow)?;
        let bytes = std::alloc::Layout::new::<[usize; 2]>()
            .extend(std::alloc::Layout::new::<ShapeNode<'_, T>>())
            .ok()
            .and_then(|(layout, _)| u64::try_from(layout.pad_to_align().size()).ok())
            .ok_or(ReductionShapeCostDenial::CapacityOverflow)?;
        let memory = self.policy.reserve_memory(bytes)?;
        self.policy.check_live()?;
        Ok(ShapeLink::Edited(Arc::new(ShapeNode {
            id,
            left,
            right,
            span,
            _memory: memory,
        })))
    }
    fn contains(
        &mut self,
        root: &ShapeLink<'a, T>,
        id: PartitionIdentity,
    ) -> Result<bool, ReductionShapeCostDenial> {
        let mut next = root.clone();
        loop {
            self.inspect()?;
            let Some((current, left, right)) = self.parts(&next) else {
                return Ok(false);
            };
            if id == current {
                return Ok(true);
            }
            next = if id < current { left } else { right };
        }
    }
    fn spine(
        &mut self,
        root: &ShapeLink<'a, T>,
        left: bool,
    ) -> Result<u64, ReductionShapeCostDenial> {
        let (mut next, mut count) = (root.clone(), 0_u64);
        loop {
            self.inspect()?;
            let Some((_, a, b)) = self.parts(&next) else {
                return Ok(count);
            };
            count = count
                .checked_add(1)
                .ok_or(ReductionShapeCostDenial::CapacityOverflow)?;
            next = if left { a } else { b };
        }
    }
}
impl<T, F> ReductionTree<T, F> {
    /// Full-build cost of the exact edited identity set, without leaf values,
    /// combines, a flat plan or a second canonical shape algorithm. Temporary
    /// changed-path cells admit their allocations and cannot escape this borrow.
    /// This predicts cost only; it grants no successful result or currentness.
    pub fn checked_full_build_cost_after_edits(
        &self,
        removed: &CanonicalUniqueVec<PartitionIdentity>,
        inserted: &CanonicalUniqueVec<PartitionIdentity>,
        policy: ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<(u64, u64), ReductionShapeCostDenial> {
        let mut engine = ShapeEngine {
            policy,
            borrowed: std::marker::PhantomData,
        };
        engine.inspect()?;
        let mut root = ShapeLink::borrowed(self.root.as_deref());
        for &id in removed.as_slice() {
            if inserted.as_slice().binary_search(&id).is_ok() {
                return Err(ReductionShapeCostDenial::Structural(
                    ReductionDenial::CoverageMismatch,
                ));
            }
            root = structure::delete(&root, id, &mut engine)?;
        }
        for &id in inserted.as_slice() {
            if engine.contains(&root, id)? {
                return Err(ReductionShapeCostDenial::Structural(
                    ReductionDenial::IdentityAlreadyPresent(id),
                ));
            }
            root = structure::insert(&root, id, &mut engine)?;
        }
        let count = self
            .len
            .checked_sub(removed.as_slice().len())
            .and_then(|n| n.checked_add(inserted.as_slice().len()))
            .ok_or(ReductionShapeCostDenial::CapacityOverflow)?;
        let ends = engine
            .spine(&root, true)?
            .checked_add(engine.spine(&root, false)?)
            .ok_or(ReductionShapeCostDenial::CapacityOverflow)?;
        let cost = super::full_cost::checked_cost(count, ends, root.span())
            .ok_or(ReductionShapeCostDenial::CapacityOverflow)?;
        engine.inspect()?;
        Ok(cost)
    }
}

#[cfg(test)]
mod tests;
