//! Authored bounds a workflow definition declares about its own shape and
//! lifetime.

use std::num::NonZeroU64;
use std::time::Duration;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ApplicationWorkflowComponentLimits {
    maximum_occurrences: u16,
    maximum_depth: u8,
    maximum_node_provenance: u32,
    maximum_connection_provenance: u32,
    maximum_port_provenance: u32,
}

impl ApplicationWorkflowComponentLimits {
    pub const fn new(
        maximum_occurrences: u16,
        maximum_depth: u8,
        maximum_node_provenance: u32,
        maximum_connection_provenance: u32,
        maximum_port_provenance: u32,
    ) -> Option<Self> {
        if maximum_occurrences == 0
            || maximum_depth == 0
            || maximum_node_provenance == 0
            || maximum_connection_provenance == 0
            || maximum_port_provenance == 0
        {
            None
        } else {
            Some(Self {
                maximum_occurrences,
                maximum_depth,
                maximum_node_provenance,
                maximum_connection_provenance,
                maximum_port_provenance,
            })
        }
    }

    pub const fn maximum_occurrences(self) -> u16 {
        self.maximum_occurrences
    }

    pub const fn maximum_depth(self) -> u8 {
        self.maximum_depth
    }

    pub const fn maximum_node_provenance(self) -> u32 {
        self.maximum_node_provenance
    }

    pub const fn maximum_connection_provenance(self) -> u32 {
        self.maximum_connection_provenance
    }

    pub const fn maximum_port_provenance(self) -> u32 {
        self.maximum_port_provenance
    }

    pub const fn fits_within(self, ceiling: Self) -> bool {
        self.maximum_occurrences <= ceiling.maximum_occurrences
            && self.maximum_depth <= ceiling.maximum_depth
            && self.maximum_node_provenance <= ceiling.maximum_node_provenance
            && self.maximum_connection_provenance <= ceiling.maximum_connection_provenance
            && self.maximum_port_provenance <= ceiling.maximum_port_provenance
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ApplicationWorkflowDefinitionLimits {
    maximum_nodes: u16,
    maximum_connections: u16,
    maximum_effects: u16,
    component_limits: ApplicationWorkflowComponentLimits,
    maximum_canonical_bytes: u32,
    total_deadline_milliseconds: Option<NonZeroU64>,
}

impl ApplicationWorkflowDefinitionLimits {
    pub const fn new(
        maximum_nodes: u16,
        maximum_connections: u16,
        maximum_effects: u16,
        component_limits: ApplicationWorkflowComponentLimits,
        maximum_canonical_bytes: u32,
    ) -> Option<Self> {
        if maximum_nodes == 0
            || maximum_connections == 0
            || maximum_effects == 0
            || maximum_canonical_bytes == 0
        {
            None
        } else {
            Some(Self {
                maximum_nodes,
                maximum_connections,
                maximum_effects,
                component_limits,
                maximum_canonical_bytes,
                total_deadline_milliseconds: None,
            })
        }
    }

    pub const fn maximum_nodes(self) -> u16 {
        self.maximum_nodes
    }

    pub const fn maximum_connections(self) -> u16 {
        self.maximum_connections
    }

    pub const fn maximum_effects(self) -> u16 {
        self.maximum_effects
    }

    pub const fn component_limits(self) -> ApplicationWorkflowComponentLimits {
        self.component_limits
    }

    pub const fn maximum_component_depth(self) -> u8 {
        self.component_limits.maximum_depth()
    }

    pub const fn maximum_canonical_bytes(self) -> u32 {
        self.maximum_canonical_bytes
    }

    /// Bounds every instance of this definition to `deadline` of trusted
    /// time from its start. The deadline is whole, nonzero milliseconds; a
    /// successor keeps the earlier of its source's deadline and its own.
    pub const fn with_total_deadline(self, deadline: Duration) -> Option<Self> {
        if !deadline.subsec_nanos().is_multiple_of(1_000_000)
            || deadline.as_millis() > u64::MAX as u128
        {
            return None;
        }
        match NonZeroU64::new(deadline.as_millis() as u64) {
            Some(milliseconds) => Some(Self {
                total_deadline_milliseconds: Some(milliseconds),
                ..self
            }),
            None => None,
        }
    }

    pub const fn total_deadline(self) -> Option<Duration> {
        match self.total_deadline_milliseconds {
            Some(milliseconds) => Some(Duration::from_millis(milliseconds.get())),
            None => None,
        }
    }
}
