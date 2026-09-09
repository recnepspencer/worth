use super::UiMountedLayoutRevision;
use worth_ui_host_contract::{UiSemanticSurfaceIdentity, UiSurfaceBindingGeneration};

/// Evidence that one exact surface geometry batch replaced the live layout.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiMountedLayoutCompletionReceipt {
    surface: UiSemanticSurfaceIdentity,
    binding: UiSurfaceBindingGeneration,
    graph_world: crate::mounting::UiMountedGraphWorldIdentity,
    revision: UiMountedLayoutRevision,
    changed_occurrences: usize,
    region_plan_rows_visited: usize,
    occurrence_index_rows: usize,
    occurrence_ancestry_steps: usize,
    region_index_rows: usize,
    region_lookup_steps: usize,
    seam_index_rows: usize,
    seam_adjacencies_visited: usize,
}

impl UiMountedLayoutCompletionReceipt {
    pub(crate) const fn new(
        surface: UiSemanticSurfaceIdentity,
        binding: UiSurfaceBindingGeneration,
        graph_world: crate::mounting::UiMountedGraphWorldIdentity,
        revision: UiMountedLayoutRevision,
        changed_occurrences: usize,
    ) -> Self {
        Self {
            surface,
            binding,
            graph_world,
            revision,
            changed_occurrences,
            region_plan_rows_visited: 0,
            occurrence_index_rows: 0,
            occurrence_ancestry_steps: 0,
            region_index_rows: 0,
            region_lookup_steps: 0,
            seam_index_rows: 0,
            seam_adjacencies_visited: 0,
        }
    }

    pub const fn surface(self) -> UiSemanticSurfaceIdentity {
        self.surface
    }

    pub const fn binding(self) -> UiSurfaceBindingGeneration {
        self.binding
    }

    pub const fn graph_world(self) -> crate::mounting::UiMountedGraphWorldIdentity {
        self.graph_world
    }

    pub const fn revision(self) -> UiMountedLayoutRevision {
        self.revision
    }

    pub const fn changed_occurrences(self) -> usize {
        self.changed_occurrences
    }

    pub(crate) fn with_region_plan_rows_visited(mut self, visited: usize) -> Self {
        self.region_plan_rows_visited = visited;
        self
    }

    pub const fn region_plan_rows_visited(self) -> usize {
        self.region_plan_rows_visited
    }

    pub(crate) fn with_occurrence_resolution_work(
        mut self,
        index_rows: usize,
        ancestry_steps: usize,
    ) -> Self {
        self.occurrence_index_rows = index_rows;
        self.occurrence_ancestry_steps = ancestry_steps;
        self
    }

    pub const fn occurrence_index_rows(self) -> usize {
        self.occurrence_index_rows
    }

    pub const fn occurrence_ancestry_steps(self) -> usize {
        self.occurrence_ancestry_steps
    }

    pub(crate) fn add_region_resolution_work(
        mut self,
        index_rows: usize,
        lookup_steps: usize,
    ) -> Self {
        self.region_index_rows += index_rows;
        self.region_lookup_steps += lookup_steps;
        self
    }

    pub const fn region_index_rows(self) -> usize {
        self.region_index_rows
    }

    pub const fn region_lookup_steps(self) -> usize {
        self.region_lookup_steps
    }

    pub(crate) fn with_seam_resolution_work(
        mut self,
        index_rows: usize,
        adjacencies_visited: usize,
    ) -> Self {
        self.seam_index_rows = index_rows;
        self.seam_adjacencies_visited = adjacencies_visited;
        self
    }

    pub const fn seam_index_rows(self) -> usize {
        self.seam_index_rows
    }

    pub const fn seam_adjacencies_visited(self) -> usize {
        self.seam_adjacencies_visited
    }
}
