/// Mechanical storage admission; this carries no filesystem authority.
pub trait ArtifactTreeStorageAllocator {
    type Denial;
}
