#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct UiHitTestSpatialWork {
    /// Nonzero only in explicit binding reconstruction, never ordinary delta maintenance.
    pub(in crate::mounting) reconstructed_rows: usize,
    pub(in crate::mounting) map_key_probes: usize,
    pub(in crate::mounting) map_node_copies: usize,
    pub(in crate::mounting) node_visits: usize,
    pub(in crate::mounting) node_copies: usize,
    pub(in crate::mounting) region_tests: usize,
    pub(in crate::mounting) motion_members_visited: usize,
    pub(in crate::mounting) motion_rows_projected: usize,
    pub(in crate::mounting) motion_tracks_considered: usize,
}

impl UiHitTestSpatialWork {
    pub(crate) fn merge(&mut self, other: impl Into<Self>) {
        let other = other.into();
        self.reconstructed_rows += other.reconstructed_rows;
        self.map_key_probes += other.map_key_probes;
        self.map_node_copies += other.map_node_copies;
        self.node_visits += other.node_visits;
        self.node_copies += other.node_copies;
        self.region_tests += other.region_tests;
        self.motion_members_visited += other.motion_members_visited;
        self.motion_rows_projected += other.motion_rows_projected;
        self.motion_tracks_considered += other.motion_tracks_considered;
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
