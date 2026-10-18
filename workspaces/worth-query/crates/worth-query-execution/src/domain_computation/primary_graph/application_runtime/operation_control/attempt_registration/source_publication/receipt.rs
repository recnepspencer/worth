//! Bounded read-only diagnostic values captured by the original native validation owner.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQuerySourcePublicationHeadForTest {
    pub binding: String,
    pub partition: [u8; 32],
    pub occurrence: worth_runtime_world::facade::ProductBranchIncarnation,
    pub generation: u64,
    pub slot: usize,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQuerySourcePublicationImageForTest {
    pub root_id: u64,
    pub native_commit: Option<u64>,
    pub native_position: Option<u64>,
    pub topology_generation: u64,
    pub outer_publications: Vec<Vec<WorthQuerySourcePublicationHeadForTest>>,
}
