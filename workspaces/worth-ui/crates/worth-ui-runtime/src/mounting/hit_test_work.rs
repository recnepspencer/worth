#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct UiHitTestSpatialWork {
    /// Nonzero only in explicit binding reconstruction, never ordinary delta maintenance.
    pub(in crate::mounting) reconstructed_rows: usize,
    pub(in crate::mounting) map_key_probes: usize,
    pub(in crate::mounting) map_node_copies: usize,
    pub(in crate::mounting) succession_cursor_steps: usize,
    pub(in crate::mounting) node_visits: usize,
    pub(in crate::mounting) node_copies: usize,
    pub(in crate::mounting) region_tests: usize,
    pub(in crate::mounting) motion_members_visited: usize,
    pub(in crate::mounting) motion_rows_projected: usize,
    /// Presented hit rows displaced by a committed scroll pose. Scrolled rows
    /// are counted apart from sampled Motion rows because they are paid for on
    /// the frame a pose lands rather than on the frame a sample is taken.
    pub(in crate::mounting) scroll_rows_displaced: usize,
    pub(in crate::mounting) scroll_geometry_members_visited: usize,
    pub(in crate::mounting) scroll_geometry_regions_visited: usize,
    pub(in crate::mounting) motion_tracks_considered: usize,
    pub(in crate::mounting) portal_rows_considered: usize,
}

impl UiHitTestSpatialWork {
    pub const fn succession_cursor_steps(self) -> usize {
        self.succession_cursor_steps
    }
    pub const fn reconstructed_rows(self) -> usize {
        self.reconstructed_rows
    }
    pub const fn map_key_probes(self) -> usize {
        self.map_key_probes
    }
    pub const fn map_node_copies(self) -> usize {
        self.map_node_copies
    }
    pub const fn node_visits(self) -> usize {
        self.node_visits
    }
    pub const fn node_copies(self) -> usize {
        self.node_copies
    }
    pub const fn region_tests(self) -> usize {
        self.region_tests
    }
    pub const fn motion_members_visited(self) -> usize {
        self.motion_members_visited
    }
    pub const fn motion_rows_projected(self) -> usize {
        self.motion_rows_projected
    }
    pub const fn scroll_rows_displaced(self) -> usize {
        self.scroll_rows_displaced
    }
    pub const fn scroll_geometry_members_visited(self) -> usize {
        self.scroll_geometry_members_visited
    }
    pub const fn scroll_geometry_regions_visited(self) -> usize {
        self.scroll_geometry_regions_visited
    }
    pub const fn motion_tracks_considered(self) -> usize {
        self.motion_tracks_considered
    }
    pub const fn portal_rows_considered(self) -> usize {
        self.portal_rows_considered
    }

    pub(crate) fn merge(&mut self, other: impl Into<Self>) {
        let other = other.into();
        self.reconstructed_rows += other.reconstructed_rows;
        self.map_key_probes += other.map_key_probes;
        self.map_node_copies += other.map_node_copies;
        self.succession_cursor_steps += other.succession_cursor_steps;
        self.node_visits += other.node_visits;
        self.node_copies += other.node_copies;
        self.region_tests += other.region_tests;
        self.motion_members_visited += other.motion_members_visited;
        self.motion_rows_projected += other.motion_rows_projected;
        self.scroll_rows_displaced += other.scroll_rows_displaced;
        self.scroll_geometry_members_visited += other.scroll_geometry_members_visited;
        self.scroll_geometry_regions_visited += other.scroll_geometry_regions_visited;
        self.motion_tracks_considered += other.motion_tracks_considered;
        self.portal_rows_considered += other.portal_rows_considered;
    }
}

impl From<super::spatial_index::UiMountedSpatialWork> for UiHitTestSpatialWork {
    fn from(work: super::spatial_index::UiMountedSpatialWork) -> Self {
        Self {
            reconstructed_rows: work.reconstructed_rows,
            map_key_probes: work.map_key_probes,
            map_node_copies: work.map_node_copies,
            node_visits: work.node_visits,
            node_copies: work.node_copies,
            region_tests: work.region_tests,
            ..Self::default()
        }
    }
}
