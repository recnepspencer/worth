//! Finite build scratch and isolated edits of immutable retained node pages.
use super::{WorthQueryManagedIndexControl, WorthQueryManagedIndexStop as Stop};
use crate::domain_computation::provider_session::WorthQueryGraphWorkSessionIdentity;
use std::{cell::Cell, sync::Arc};
use worth_execution::{
    ExecutionArray, ExecutionOrderedMap, ExecutionRequest, ExecutionRequestArrayBuilder,
};
mod lineage;
mod pages;
use lineage::NodeLineage;
use pages::{NodePages, PageEdit};

/// A node coordinate issued by one genuine provider-build session.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryManagedIndexNodeId {
    pub(super) owner: WorthQueryGraphWorkSessionIdentity,
    pub(super) ordinal: usize,
}

/// Mutable finite build storage. Only the initialized prefix can be sealed.
pub struct WorthQueryManagedIndexNodeWriter<'a, Node: Copy> {
    owner: WorthQueryGraphWorkSessionIdentity,
    lineage: Arc<NodeLineage>,
    storage: Writing<'a, Node>,
    written: usize,
    live: usize,
    capacity: usize,
    source_count: usize,
    changed: Option<ExecutionOrderedMap<usize, ()>>,
    before_sources: Option<&'a super::source_structure::SourceTree>,
    control: &'a WorthQueryManagedIndexControl<'a>,
}

enum Writing<'a, Node> {
    Build(ExecutionArray<Cell<Option<Node>>>),
    Edit { pages: PageEdit<'a, Node> },
}

/// Sealed nodes retained by the registered root's actual original allocation.
pub struct WorthQueryManagedIndexNodes<Node> {
    owner: WorthQueryGraphWorkSessionIdentity,
    lineage: Arc<NodeLineage>,
    pages: NodePages<Node>,
    frontier: usize,
    live: usize,
    root: Option<WorthQueryManagedIndexNodeId>,
    changed: Option<ExecutionOrderedMap<usize, ()>>,
}

impl<'a, Node: Copy> WorthQueryManagedIndexNodeWriter<'a, Node> {
    pub(super) fn allocate(
        execution: &ExecutionRequest<'_, '_>,
        owner: WorthQueryGraphWorkSessionIdentity,
        capacity: usize,
        source_count: usize,
        control: &'a WorthQueryManagedIndexControl<'a>,
    ) -> Result<Self, Stop> {
        let lineage = NodeLineage::retain(execution, control)?;
        let mut storage = ExecutionRequestArrayBuilder::allocate(*execution, capacity)
            .map_err(|cause| control.refuse(Stop::Storage(cause)))?;
        for _ in 0..capacity {
            control.checkpoint(1)?;
            storage
                .push(Cell::new(None))
                .map_err(|cause| control.refuse(Stop::Storage(cause)))?;
        }
        let scratch = storage
            .seal()
            .map_err(|cause| control.refuse(Stop::Storage(cause)))?;
        Ok(Self {
            owner,
            lineage,
            storage: Writing::Build(scratch),
            written: 0,
            live: 0,
            capacity,
            source_count,
            changed: None,
            before_sources: None,
            control,
        })
    }
    pub fn len(&self) -> usize {
        self.live
    }
    pub fn is_empty(&self) -> bool {
        self.live == 0
    }
    pub fn push(&mut self, node: Node) -> Result<WorthQueryManagedIndexNodeId, Stop> {
        self.control.checkpoint(1)?;
        if self.live == self.capacity {
            return Err(self.control.refuse(Stop::NodeCapacityExceeded));
        }
        let ordinal = self.lineage.issue(self.control)?;
        let next = ordinal
            .checked_add(1)
            .ok_or_else(|| self.control.refuse(Stop::CapacityOverflow))?;
        self.write(ordinal, Some(node))?;
        self.written = next;
        self.live += 1;
        self.control.poll()?;
        Ok(WorthQueryManagedIndexNodeId {
            owner: self.owner,
            ordinal,
        })
    }
    pub fn get(&self, id: WorthQueryManagedIndexNodeId) -> Result<Node, Stop> {
        self.control.checkpoint(1)?;
        self.validate(id)?;
        match &self.storage {
            Writing::Build(scratch) => scratch.get(id.ordinal).and_then(Cell::get),
            Writing::Edit { pages } => pages.get(id.ordinal, self.control)?,
        }
        .ok_or_else(|| self.control.refuse(Stop::UnwrittenNode))
    }
    pub fn replace(&mut self, id: WorthQueryManagedIndexNodeId, node: Node) -> Result<(), Stop> {
        self.control.checkpoint(1)?;
        self.get(id)?;
        self.write(id.ordinal, Some(node))?;
        self.control.poll()
    }

    /// Retire a node coordinate without making it name a subsequent insertion.
    pub fn remove(&mut self, id: WorthQueryManagedIndexNodeId) -> Result<(), Stop> {
        self.get(id)?;
        self.write(id.ordinal, None)?;
        self.live -= 1;
        self.control.poll()
    }

    /// Locate this occurrence's node in the checked predecessor structure.
    /// The coordinate is descriptive; `get` checks its current writer liveness.
    /// Cold and opaque writers have no predecessor Source correspondence.
    pub fn before_source_node(
        &self,
        slot: super::WorthQueryManagedIndexSourceSlot,
    ) -> Result<WorthQueryManagedIndexNodeId, Stop> {
        let tree = self
            .before_sources
            .ok_or_else(|| self.control.refuse(Stop::SourceStructureUnavailable))?;
        let ordinal = tree
            .source_node(slot, self.control)?
            .ok_or_else(|| self.control.refuse(Stop::SourceSlotUnavailable))?;
        let id = WorthQueryManagedIndexNodeId {
            owner: self.owner,
            ordinal,
        };
        self.get(id)?;
        Ok(id)
    }

    pub(super) fn track_changes(&mut self, before: &'a super::source_structure::SourceTree) {
        self.changed = Some(ExecutionOrderedMap::default());
        self.before_sources = Some(before);
    }

    fn write(&mut self, ordinal: usize, value: Option<Node>) -> Result<(), Stop> {
        if let Some(changed) = &mut self.changed {
            let Writing::Edit { pages } = &self.storage else {
                unreachable!("only retained edits track changed coordinates");
            };
            super::source_structure::record_change(
                changed,
                ordinal,
                &pages.execution(),
                self.control,
            )?;
        }
        match &mut self.storage {
            Writing::Build(scratch) => {
                let cell = scratch
                    .get(ordinal)
                    .ok_or_else(|| self.control.refuse(Stop::NodeCapacityExceeded))?;
                cell.set(value);
                Ok(())
            }
            Writing::Edit { pages } => pages.replace(ordinal, value, self.control),
        }
    }
    fn validate(&self, id: WorthQueryManagedIndexNodeId) -> Result<(), Stop> {
        if id.owner != self.owner {
            return Err(self.control.refuse(Stop::ForeignNode));
        }
        if id.ordinal >= self.written {
            return Err(self.control.refuse(Stop::UnwrittenNode));
        }
        Ok(())
    }
    pub(super) fn seal(
        self,
        execution: &ExecutionRequest<'_, '_>,
        root: Option<WorthQueryManagedIndexNodeId>,
    ) -> Result<WorthQueryManagedIndexNodes<Node>, Stop> {
        self.control.poll()?;
        match (self.source_count, root) {
            (0, None) if self.live == 0 => {}
            (0, _) | (_, None) => return Err(self.control.refuse(Stop::InvalidRoot)),
            (_, Some(root)) => {
                self.get(root)?;
            }
        }
        let pages = match self.storage {
            Writing::Build(scratch) => NodePages::from_scratch(
                execution,
                &scratch.elements()[..self.written],
                self.control,
            )?,
            Writing::Edit { pages } => pages.seal(self.control)?,
        };
        self.control.poll()?;
        Ok(WorthQueryManagedIndexNodes {
            owner: self.owner,
            lineage: self.lineage,
            pages,
            frontier: self.written,
            live: self.live,
            root,
            changed: self.changed,
        })
    }
}

impl<Node: Copy> WorthQueryManagedIndexNodes<Node> {
    pub(super) fn frontier(&self) -> usize {
        self.frontier
    }
    pub(super) fn node_at_ordinal(
        &self,
        ordinal: usize,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<Option<&Node>, Stop> {
        self.pages.get(ordinal, control)
    }
    pub(super) fn take_changes(&mut self) -> Option<ExecutionOrderedMap<usize, ()>> {
        self.changed.take()
    }

    pub fn len(&self) -> usize {
        self.live
    }
    pub fn is_empty(&self) -> bool {
        self.live == 0
    }
    pub fn root(&self) -> Option<WorthQueryManagedIndexNodeId> {
        self.root
    }
    pub fn get(
        &self,
        id: WorthQueryManagedIndexNodeId,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<&Node, Stop> {
        control.checkpoint(1)?;
        if id.owner != self.owner {
            return Err(control.refuse(Stop::ForeignNode));
        }
        self.pages
            .get(id.ordinal, control)?
            .ok_or_else(|| control.refuse(Stop::UnwrittenNode))
    }

    /// Framework-only: a fresh original request owns every new page. The retained
    /// root holds backing, never the authority of the request that constructed it.
    pub(super) fn edit<'a>(
        &self,
        execution: ExecutionRequest<'a, 'a>,
        capacity: usize,
        source_count: usize,
        control: &'a WorthQueryManagedIndexControl<'a>,
    ) -> Result<WorthQueryManagedIndexNodeWriter<'a, Node>, Stop> {
        control.checkpoint(1)?;
        if capacity < self.live {
            return Err(control.refuse(Stop::NodeCapacityExceeded));
        }
        Ok(WorthQueryManagedIndexNodeWriter {
            owner: self.owner,
            lineage: Arc::clone(&self.lineage),
            storage: Writing::Edit {
                pages: PageEdit::new(self.pages.clone(), execution),
            },
            written: self.frontier,
            live: self.live,
            capacity,
            source_count,
            changed: None,
            before_sources: None,
            control,
        })
    }
}
