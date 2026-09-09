mod bounds;
mod mutation;
mod query;
mod tree;

use bounds::Bounds;
use std::rc::Rc;
use tree::{Key, Link, Node};
use worth_ui_host_contract::UiMountedInstanceIdentity;

/// Reconstructible spatial acceleration for one coordinate space and binding.
/// It carries identities, never receipts or interaction authority.
#[derive(Clone, Default)]
pub(in crate::mounting) struct UiMountedSpatialTree {
    root: Link,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct UiMountedSpatialWork {
    /// Nonzero only in explicit binding reconstruction, never ordinary delta maintenance.
    pub(in crate::mounting) reconstructed_rows: usize,
    pub(in crate::mounting) map_key_probes: usize,
    pub(in crate::mounting) map_node_copies: usize,
    pub(in crate::mounting) node_visits: usize,
    pub(in crate::mounting) node_copies: usize,
    pub(in crate::mounting) region_tests: usize,
}

impl UiMountedSpatialWork {
    pub(crate) fn merge(&mut self, other: Self) {
        self.reconstructed_rows += other.reconstructed_rows;
        self.map_key_probes += other.map_key_probes;
        self.map_node_copies += other.map_node_copies;
        self.node_visits += other.node_visits;
        self.node_copies += other.node_copies;
        self.region_tests += other.region_tests;
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::mounting) struct UiMountedSpatialBudget {
    pub(in crate::mounting) node_visits: usize,
    pub(in crate::mounting) candidates: usize,
}

#[derive(Debug, Eq, PartialEq)]
pub(in crate::mounting) enum UiMountedSpatialQueryDenial {
    InvalidGeometry,
    NodeBudget { work: UiMountedSpatialWork },
    CandidateBudget { work: UiMountedSpatialWork },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::mounting) enum UiMountedSpatialMutationDenial {
    InvalidBounds,
}

pub(in crate::mounting) struct UiMountedSpatialQuery {
    pub(in crate::mounting) instances: Vec<UiMountedInstanceIdentity>,
    pub(in crate::mounting) work: UiMountedSpatialWork,
}

impl UiMountedSpatialTree {
    pub(in crate::mounting) fn is_empty(&self) -> bool {
        self.root.is_none()
    }

    /// Logical allocation bytes, excluding allocator overhead and cross-version sharing.
    pub(in crate::mounting) fn retained_structural_bytes(&self) -> Option<usize> {
        let count = self.root.as_ref().map_or(0, |root| root.len);
        Self::structural_bytes_for_nodes(count)
    }

    pub(in crate::mounting) fn structural_bytes_for_nodes(count: usize) -> Option<usize> {
        count.checked_mul(std::mem::size_of::<Node>() + 2 * std::mem::size_of::<usize>())
    }

    /// Bounds are min-x, min-y, max-x, max-y in the caller's single admitted space.
    /// Empty regions are omitted; non-finite or reversed bounds are rejected.
    /// The owning source supplies its exact predecessor; this acceleration tree
    /// does not own an independent instance-to-geometry truth table.
    pub(in crate::mounting) fn replace(
        &mut self,
        instance: UiMountedInstanceIdentity,
        previous: Option<[f64; 4]>,
        successor: Option<[f64; 4]>,
    ) -> Result<UiMountedSpatialWork, UiMountedSpatialMutationDenial> {
        let previous = previous
            .map(Bounds::admit)
            .transpose()
            .map_err(|_| UiMountedSpatialMutationDenial::InvalidBounds)?
            .flatten();
        let successor = successor
            .map(Bounds::admit)
            .transpose()
            .map_err(|_| UiMountedSpatialMutationDenial::InvalidBounds)?
            .flatten();
        let mut work = UiMountedSpatialWork::default();
        if previous == successor {
            return Ok(work);
        }
        if let Some(previous) = previous {
            self.root = mutation::remove(&self.root, Key::new(instance, previous), &mut work);
        }
        if let Some(successor) = successor {
            self.root = Some(mutation::insert(
                &self.root,
                Key::new(instance, successor),
                successor,
                &mut work,
            ));
        }
        Ok(work)
    }

    pub(in crate::mounting) fn at_point(
        &self,
        point: [f64; 2],
        budget: UiMountedSpatialBudget,
    ) -> Result<UiMountedSpatialQuery, UiMountedSpatialQueryDenial> {
        query::at_point(&self.root, point, budget)
    }

    /// Finds half-open rectangle intersections, without treating candidates as
    /// painted coverage, hit targets, or ordering authority.
    pub(in crate::mounting) fn intersecting(
        &self,
        bounds: [f64; 4],
        budget: UiMountedSpatialBudget,
    ) -> Result<UiMountedSpatialQuery, UiMountedSpatialQueryDenial> {
        query::intersecting(&self.root, bounds, budget)
    }

    /// Node payload bytes, excluding reference-count control blocks and allocator overhead.
    pub(in crate::mounting) fn retained_node_payload_bytes(&self) -> Option<usize> {
        self.root
            .as_ref()
            .map_or(0, |root| root.len)
            .checked_mul(std::mem::size_of::<Node>())
    }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod rectangle_tests;
