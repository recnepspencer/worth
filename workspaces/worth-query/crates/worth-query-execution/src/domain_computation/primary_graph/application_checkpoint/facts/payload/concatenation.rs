//! Complete native read roots append through admitted, balanced path copies.
use super::{FactPayload, FactStorage, StorageControl};
use crate::domain_computation::primary_graph::{
    application_attempt::retained_decision_facts::StoreDenial, fact_verification_observation,
};
use worth_execution::ExecutionArrayBuilder;

impl FactPayload {
    /// Append preserves both original backings and their complete reservations.
    /// Only complete roots enter here; range views remain descriptive accessors.
    pub(in crate::domain_computation::primary_graph) fn append_completed(
        &self,
        tail: Self,
        control: StorageControl<'_, '_>,
    ) -> Result<Self, StoreDenial> {
        control.check_live()?;
        self.len()
            .checked_add(tail.len())
            .ok_or(StoreDenial::Representability)?;
        join(
            self.normalized(control)?,
            tail.normalized(control)?,
            control,
        )
    }

    /// A clipped joined range is reduced to its retained leaf/path backing.
    /// It never wraps the old root, so repeated projection cannot retain a
    /// chain of obsolete aggregate roots or invent a tall atomic leaf.
    fn normalized(&self, control: StorageControl<'_, '_>) -> Result<Self, StoreDenial> {
        control.check_live()?;
        let FactStorage::Joined { children } = &self.storage else {
            return Ok(self.clone());
        };
        if self.start == 0 && self.length == children[0].len() + children[1].len() {
            return Ok(self.clone());
        }
        let end = self.start + self.length;
        let middle = children[0].len();
        if end <= middle {
            return children[0].slice(self.start..end).normalized(control);
        }
        if self.start >= middle {
            return children[1]
                .slice(self.start - middle..end - middle)
                .normalized(control);
        }
        join(
            children[0].slice(self.start..middle).normalized(control)?,
            children[1].slice(0..end - middle).normalized(control)?,
            control,
        )
    }

    fn complete_children(&self) -> Result<(&Self, &Self), StoreDenial> {
        let FactStorage::Joined { children } = &self.storage else {
            return Err(StoreDenial::ConflictingBody);
        };
        if self.start != 0 || self.length != children[0].len() + children[1].len() {
            return Err(StoreDenial::ConflictingBody);
        }
        Ok((&children[0], &children[1]))
    }
}

fn node(
    left: FactPayload,
    right: FactPayload,
    control: StorageControl<'_, '_>,
) -> Result<FactPayload, StoreDenial> {
    control.check_live()?;
    if left.height.max(right.height) >= usize::BITS as usize - 1 {
        return Err(StoreDenial::Representability);
    }
    let mut children = ExecutionArrayBuilder::allocate(2, control.policy())?;
    children.push(left)?;
    children.push(right)?;
    control.check_live()?;
    let joined = FactPayload::joined(children.seal()?);
    fact_verification_observation::completed_fact_join();
    Ok(joined)
}

fn join(
    left: FactPayload,
    right: FactPayload,
    control: StorageControl<'_, '_>,
) -> Result<FactPayload, StoreDenial> {
    control.check_live()?;
    if left.is_empty() {
        return Ok(right);
    }
    if right.is_empty() {
        return Ok(left);
    }
    if left.height > right.height + 1 {
        let (a, b) = left.complete_children()?;
        return balance(a.clone(), join(b.clone(), right, control)?, control);
    }
    if right.height > left.height + 1 {
        let (a, b) = right.complete_children()?;
        return balance(join(left, a.clone(), control)?, b.clone(), control);
    }
    node(left, right, control)
}

fn balance(
    left: FactPayload,
    right: FactPayload,
    control: StorageControl<'_, '_>,
) -> Result<FactPayload, StoreDenial> {
    if left.height > right.height + 1 {
        let (a, b) = left.complete_children()?;
        if a.height >= b.height {
            return node(a.clone(), node(b.clone(), right, control)?, control);
        }
        let (ba, bb) = b.complete_children()?;
        return node(
            node(a.clone(), ba.clone(), control)?,
            node(bb.clone(), right, control)?,
            control,
        );
    }
    if right.height > left.height + 1 {
        let (a, b) = right.complete_children()?;
        if b.height >= a.height {
            return node(node(left, a.clone(), control)?, b.clone(), control);
        }
        let (aa, ab) = a.complete_children()?;
        return node(
            node(left, aa.clone(), control)?,
            node(ab.clone(), b.clone(), control)?,
            control,
        );
    }
    node(left, right, control)
}

#[cfg(test)]
mod tests;
