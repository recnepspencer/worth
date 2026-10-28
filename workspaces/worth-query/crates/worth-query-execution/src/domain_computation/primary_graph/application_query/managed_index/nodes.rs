//! Finite local scratch and separately admitted immutable retained node storage.
use super::{WorthQueryManagedIndexControl, WorthQueryManagedIndexStop as Stop};
use crate::domain_computation::provider_session::WorthQueryGraphWorkSessionIdentity;
use std::cell::Cell;
use worth_execution::{ExecutionArray, ExecutionRequest, ExecutionRequestArrayBuilder};

/// A node coordinate issued by one genuine provider-build session.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryManagedIndexNodeId {
    owner: WorthQueryGraphWorkSessionIdentity,
    ordinal: usize,
}

/// Mutable finite build storage. Only the initialized prefix can be sealed.
pub struct WorthQueryManagedIndexNodeWriter<'a, Node: Copy> {
    owner: WorthQueryGraphWorkSessionIdentity,
    scratch: ExecutionArray<Cell<Option<Node>>>,
    written: usize,
    source_count: usize,
    control: &'a WorthQueryManagedIndexControl<'a>,
}

/// Sealed nodes retained by the registered root's actual original allocation.
pub struct WorthQueryManagedIndexNodes<Node> {
    owner: WorthQueryGraphWorkSessionIdentity,
    nodes: ExecutionArray<Node>,
    root: Option<WorthQueryManagedIndexNodeId>,
}

impl<'a, Node: Copy> WorthQueryManagedIndexNodeWriter<'a, Node> {
    pub(super) fn allocate(
        execution: &ExecutionRequest<'_, '_>,
        owner: WorthQueryGraphWorkSessionIdentity,
        capacity: usize,
        source_count: usize,
        control: &'a WorthQueryManagedIndexControl<'a>,
    ) -> Result<Self, Stop> {
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
            scratch,
            written: 0,
            source_count,
            control,
        })
    }
    pub fn len(&self) -> usize {
        self.written
    }
    pub fn is_empty(&self) -> bool {
        self.written == 0
    }
    pub fn push(&mut self, node: Node) -> Result<WorthQueryManagedIndexNodeId, Stop> {
        self.control.checkpoint(1)?;
        if self.written == self.scratch.len() {
            return Err(self.control.refuse(Stop::NodeCapacityExceeded));
        }
        let ordinal = self.written;
        self.scratch[ordinal].set(Some(node));
        self.written += 1;
        self.control.poll()?;
        Ok(WorthQueryManagedIndexNodeId {
            owner: self.owner,
            ordinal,
        })
    }
    pub fn get(&self, id: WorthQueryManagedIndexNodeId) -> Result<Node, Stop> {
        self.control.checkpoint(1)?;
        self.validate(id)?;
        self.scratch[id.ordinal]
            .get()
            .ok_or_else(|| self.control.refuse(Stop::UnwrittenNode))
    }
    pub fn replace(&mut self, id: WorthQueryManagedIndexNodeId, node: Node) -> Result<(), Stop> {
        self.control.checkpoint(1)?;
        self.validate(id)?;
        self.scratch[id.ordinal].set(Some(node));
        self.control.poll()
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
            (0, None) if self.written == 0 => {}
            (0, _) | (_, None) => return Err(self.control.refuse(Stop::InvalidRoot)),
            (_, Some(root)) => self.validate(root)?,
        }
        let mut retained = ExecutionRequestArrayBuilder::allocate(*execution, self.written)
            .map_err(|cause| self.control.refuse(Stop::Storage(cause)))?;
        for index in 0..self.written {
            self.control.checkpoint(1)?;
            let node = self.scratch[index]
                .get()
                .ok_or_else(|| self.control.refuse(Stop::UnwrittenNode))?;
            retained
                .push(node)
                .map_err(|cause| self.control.refuse(Stop::Storage(cause)))?;
        }
        let nodes = retained
            .seal()
            .map_err(|cause| self.control.refuse(Stop::Storage(cause)))?;
        self.control.poll()?;
        Ok(WorthQueryManagedIndexNodes {
            owner: self.owner,
            nodes,
            root,
        })
    }
}

impl<Node> WorthQueryManagedIndexNodes<Node> {
    pub fn len(&self) -> usize {
        self.nodes.len()
    }
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
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
        self.nodes
            .get(id.ordinal)
            .ok_or_else(|| control.refuse(Stop::UnwrittenNode))
    }
}
