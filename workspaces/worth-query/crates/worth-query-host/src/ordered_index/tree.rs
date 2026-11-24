//! Height-balanced AVL tree mechanics with associative summary propagation over
//! Query's isolated `WorthQueryManagedIndexNodeWriter`.
//!
//! Every descent, comparison, rotation, and summary fold charges `control.checkpoint(1)`
//! before doing work and allocates zero auxiliary heap collections; failed staged edits
//! are discarded atomically by Query without affecting the retained root.

use super::{
    WorthQueryOrderedIndexDomain, WorthQueryOrderedIndexNode, WorthQueryOrderedIndexSources,
};
use std::cmp::Ordering;
use worth_query_declaration::facade::{
    application_query::ApplicationManagedIndexBinding, application_schema::ApplicationSchema,
};
use worth_query_execution::facade::primary_graph::{
    WorthQueryManagedIndexControl, WorthQueryManagedIndexNodeId, WorthQueryManagedIndexNodeWriter,
    WorthQueryManagedIndexProviderStop, WorthQueryManagedIndexSourceRows,
    WorthQueryManagedIndexSourceSlot, WorthQueryManagedIndexStop,
};

const MAX_AVL_DEPTH: u32 = 2 * usize::BITS;

type Stop<D> = WorthQueryManagedIndexProviderStop<D>;
type Node<D> = WorthQueryOrderedIndexNode<D>;

/// Folds `[own, left, right]` summaries in canonical order using `domain.combine_summaries`.
/// Used identically by tree maintenance and Query's postorder `merge_summaries` verifier.
/// Complexity: O(1) combine calls (at most 2).
pub(super) fn fold_summaries<S, I, D>(
    domain: &D,
    parts: [Option<&D::Summary>; 3],
    sources: &WorthQueryOrderedIndexSources<'_, '_, S, I>,
    control: &WorthQueryManagedIndexControl<'_>,
) -> Result<D::Summary, Stop<D::Denial>>
where
    S: ApplicationSchema,
    I: ApplicationManagedIndexBinding<S>,
    D: WorthQueryOrderedIndexDomain<S, I>,
{
    let mut combined: Option<D::Summary> = None;
    for part in parts.into_iter().flatten() {
        control.checkpoint(1)?;
        combined = Some(match combined {
            None => part.clone(),
            Some(ref current) => domain.combine_summaries(current, part, sources, control)?,
        });
    }
    combined.ok_or_else(|| {
        control
            .refuse(WorthQueryManagedIndexStop::SummaryUnavailable)
            .into()
    })
}

pub(super) struct TreeContext<'call, 'source, 'node, 'control, S, I, D>
where
    S: ApplicationSchema,
    I: ApplicationManagedIndexBinding<S>,
    D: WorthQueryOrderedIndexDomain<S, I>,
{
    sources: &'call WorthQueryManagedIndexSourceRows<'source, S, I>,
    nodes: &'call mut WorthQueryManagedIndexNodeWriter<'node, Node<D::Summary>>,
    domain: &'call D,
    control: &'call WorthQueryManagedIndexControl<'control>,
}

impl<'call, 'source, 'node, 'control, S, I, D> TreeContext<'call, 'source, 'node, 'control, S, I, D>
where
    S: ApplicationSchema,
    I: ApplicationManagedIndexBinding<S>,
    D: WorthQueryOrderedIndexDomain<S, I>,
{
    pub(super) fn new(
        sources: &'call WorthQueryManagedIndexSourceRows<'source, S, I>,
        nodes: &'call mut WorthQueryManagedIndexNodeWriter<'node, Node<D::Summary>>,
        domain: &'call D,
        control: &'call WorthQueryManagedIndexControl<'control>,
    ) -> Self {
        Self {
            sources,
            nodes,
            domain,
            control,
        }
    }

    /// Inserts one genuine source slot into the AVL tree in O(log N) steps.
    pub(super) fn insert(
        &mut self,
        root: Option<WorthQueryManagedIndexNodeId>,
        slot: WorthQueryManagedIndexSourceSlot,
    ) -> Result<WorthQueryManagedIndexNodeId, Stop<D::Denial>> {
        let own_summary =
            self.domain
                .source_summary(slot, self.sources.get(slot)?, self.control)?;
        self.insert_at(root, slot, own_summary, 0)
    }

    /// Removes one genuine source slot from the AVL tree in O(log N) steps.
    pub(super) fn remove(
        &mut self,
        root: Option<WorthQueryManagedIndexNodeId>,
        slot: WorthQueryManagedIndexSourceSlot,
    ) -> Result<Option<WorthQueryManagedIndexNodeId>, Stop<D::Denial>> {
        self.remove_at(root, slot, 0)
    }
}

impl<S, I, D> TreeContext<'_, '_, '_, '_, S, I, D>
where
    S: ApplicationSchema,
    I: ApplicationManagedIndexBinding<S>,
    D: WorthQueryOrderedIndexDomain<S, I>,
{
    fn compare_slots(
        &self,
        left: WorthQueryManagedIndexSourceSlot,
        right: WorthQueryManagedIndexSourceSlot,
    ) -> Result<Ordering, Stop<D::Denial>> {
        self.control.checkpoint(1)?;
        if left == right {
            return Ok(Ordering::Equal);
        }
        let left_row = self.sources.get(left)?;
        let right_row = self.sources.get(right)?;
        let order = self
            .domain
            .compare_rows(left_row, right_row, self.control)?;
        Ok(order.then_with(|| left.cmp(&right)))
    }

    fn insert_at(
        &mut self,
        root: Option<WorthQueryManagedIndexNodeId>,
        slot: WorthQueryManagedIndexSourceSlot,
        own_summary: D::Summary,
        depth: u32,
    ) -> Result<WorthQueryManagedIndexNodeId, Stop<D::Denial>> {
        self.control.checkpoint(1)?;
        if depth > MAX_AVL_DEPTH {
            return Err(self
                .control
                .refuse(WorthQueryManagedIndexStop::SourceStructureCycle)
                .into());
        }
        let Some(id) = root else {
            return Ok(self.nodes.push(WorthQueryOrderedIndexNode {
                source: slot,
                summary: own_summary.clone(),
                own_summary,
                left: None,
                right: None,
                height: 1,
            })?);
        };
        let mut node = self.nodes.get(id)?.clone();
        match self.compare_slots(slot, node.source)? {
            Ordering::Less => {
                node.left = Some(self.insert_at(node.left, slot, own_summary, depth + 1)?);
            }
            Ordering::Greater => {
                node.right = Some(self.insert_at(node.right, slot, own_summary, depth + 1)?);
            }
            Ordering::Equal => {
                return Err(self
                    .control
                    .refuse(WorthQueryManagedIndexStop::DuplicateSourceSlot)
                    .into());
            }
        }
        self.refresh_node(id, node)?;
        self.balance(id)
    }

    fn remove_at(
        &mut self,
        root: Option<WorthQueryManagedIndexNodeId>,
        slot: WorthQueryManagedIndexSourceSlot,
        depth: u32,
    ) -> Result<Option<WorthQueryManagedIndexNodeId>, Stop<D::Denial>> {
        self.control.checkpoint(1)?;
        if depth > MAX_AVL_DEPTH {
            return Err(self
                .control
                .refuse(WorthQueryManagedIndexStop::SourceStructureCycle)
                .into());
        }
        let id = root.ok_or_else(|| {
            self.control
                .refuse(WorthQueryManagedIndexStop::SourceStructureIncomplete)
        })?;
        let (node_source, node_left, node_right) = {
            let guard = self.nodes.get(id)?;
            (guard.source, guard.left, guard.right)
        };
        match self.compare_slots(slot, node_source)? {
            Ordering::Less => {
                let mut node = self.nodes.get(id)?.clone();
                node.left = self.remove_at(node_left, slot, depth + 1)?;
                self.refresh_node(id, node)?;
                self.balance(id).map(Some)
            }
            Ordering::Greater => {
                let mut node = self.nodes.get(id)?.clone();
                node.right = self.remove_at(node_right, slot, depth + 1)?;
                self.refresh_node(id, node)?;
                self.balance(id).map(Some)
            }
            Ordering::Equal => match (node_left, node_right) {
                (None, child) | (child, None) => {
                    self.nodes.remove(id)?;
                    Ok(child)
                }
                (Some(_), Some(right)) => {
                    let mut node = self.nodes.get(id)?.clone();
                    let (succ_slot, succ_summary, right) = self.remove_minimum(right, depth + 1)?;
                    node.source = succ_slot;
                    node.own_summary = succ_summary;
                    node.right = right;
                    self.refresh_node(id, node)?;
                    self.balance(id).map(Some)
                }
            },
        }
    }

    fn remove_minimum(
        &mut self,
        id: WorthQueryManagedIndexNodeId,
        depth: u32,
    ) -> Result<
        (
            WorthQueryManagedIndexSourceSlot,
            D::Summary,
            Option<WorthQueryManagedIndexNodeId>,
        ),
        Stop<D::Denial>,
    > {
        self.control.checkpoint(1)?;
        if depth > MAX_AVL_DEPTH {
            return Err(self
                .control
                .refuse(WorthQueryManagedIndexStop::SourceStructureCycle)
                .into());
        }
        let left = self.nodes.get(id)?.left;
        let Some(left) = left else {
            let (source, own_summary, right) = {
                let guard = self.nodes.get(id)?;
                (guard.source, guard.own_summary.clone(), guard.right)
            };
            self.nodes.remove(id)?;
            return Ok((source, own_summary, right));
        };
        let mut node = self.nodes.get(id)?.clone();
        let (source, own_summary, left) = self.remove_minimum(left, depth + 1)?;
        node.left = left;
        self.refresh_node(id, node)?;
        Ok((source, own_summary, Some(self.balance(id)?)))
    }

    fn height(&self, id: Option<WorthQueryManagedIndexNodeId>) -> Result<u8, Stop<D::Denial>> {
        match id {
            Some(id) => Ok(self.nodes.get(id)?.height),
            None => Ok(0),
        }
    }

    fn refresh_node(
        &mut self,
        id: WorthQueryManagedIndexNodeId,
        mut node: Node<D::Summary>,
    ) -> Result<(), Stop<D::Denial>> {
        self.control.checkpoint(1)?;
        let (left_height, right_height, summary) = {
            let left = node.left.map(|c| self.nodes.get(c)).transpose()?;
            let right = node.right.map(|c| self.nodes.get(c)).transpose()?;
            let left_height = left.as_ref().map_or(0, |n| n.height);
            let right_height = right.as_ref().map_or(0, |n| n.height);
            let parts = [
                Some(&node.own_summary),
                left.as_deref().map(|n| &n.summary),
                right.as_deref().map(|n| &n.summary),
            ];
            let sources = WorthQueryOrderedIndexSources::from_rows(self.sources);
            let summary = fold_summaries(self.domain, parts, &sources, self.control)?;
            (left_height, right_height, summary)
        };
        node.height = left_height
            .max(right_height)
            .checked_add(1)
            .ok_or_else(|| {
                self.control
                    .refuse(WorthQueryManagedIndexStop::CapacityOverflow)
            })?;
        node.summary = summary;
        self.nodes.replace(id, node)?;
        Ok(())
    }

    fn balance(
        &mut self,
        id: WorthQueryManagedIndexNodeId,
    ) -> Result<WorthQueryManagedIndexNodeId, Stop<D::Denial>> {
        self.control.checkpoint(1)?;
        let (node_left, node_right) = {
            let guard = self.nodes.get(id)?;
            (guard.left, guard.right)
        };
        let left = self.height(node_left)?;
        let right = self.height(node_right)?;
        if i16::from(left) - i16::from(right) > 1 {
            let child_id = node_left.ok_or_else(|| {
                self.control
                    .refuse(WorthQueryManagedIndexStop::SourceStructureIncomplete)
            })?;
            let (child_left, child_right) = {
                let guard = self.nodes.get(child_id)?;
                (guard.left, guard.right)
            };
            if self.height(child_left)? < self.height(child_right)? {
                let rotated = self.rotate_left(child_id)?;
                let mut node = self.nodes.get(id)?.clone();
                node.left = Some(rotated);
                self.refresh_node(id, node)?;
            }
            self.rotate_right(id)
        } else if i16::from(right) - i16::from(left) > 1 {
            let child_id = node_right.ok_or_else(|| {
                self.control
                    .refuse(WorthQueryManagedIndexStop::SourceStructureIncomplete)
            })?;
            let (child_left, child_right) = {
                let guard = self.nodes.get(child_id)?;
                (guard.left, guard.right)
            };
            if self.height(child_right)? < self.height(child_left)? {
                let rotated = self.rotate_right(child_id)?;
                let mut node = self.nodes.get(id)?.clone();
                node.right = Some(rotated);
                self.refresh_node(id, node)?;
            }
            self.rotate_left(id)
        } else {
            Ok(id)
        }
    }

    fn rotate_left(
        &mut self,
        id: WorthQueryManagedIndexNodeId,
    ) -> Result<WorthQueryManagedIndexNodeId, Stop<D::Denial>> {
        self.control.checkpoint(1)?;
        let mut node = self.nodes.get(id)?.clone();
        let pivot_id = node.right.ok_or_else(|| {
            self.control
                .refuse(WorthQueryManagedIndexStop::SourceStructureIncomplete)
        })?;
        let mut pivot = self.nodes.get(pivot_id)?.clone();
        node.right = pivot.left;
        self.refresh_node(id, node)?;
        pivot.left = Some(id);
        self.refresh_node(pivot_id, pivot)?;
        Ok(pivot_id)
    }

    fn rotate_right(
        &mut self,
        id: WorthQueryManagedIndexNodeId,
    ) -> Result<WorthQueryManagedIndexNodeId, Stop<D::Denial>> {
        self.control.checkpoint(1)?;
        let mut node = self.nodes.get(id)?.clone();
        let pivot_id = node.left.ok_or_else(|| {
            self.control
                .refuse(WorthQueryManagedIndexStop::SourceStructureIncomplete)
        })?;
        let mut pivot = self.nodes.get(pivot_id)?.clone();
        node.left = pivot.right;
        self.refresh_node(id, node)?;
        pivot.right = Some(id);
        self.refresh_node(pivot_id, pivot)?;
        Ok(pivot_id)
    }
}
