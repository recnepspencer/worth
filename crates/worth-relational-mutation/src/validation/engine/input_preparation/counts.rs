#[derive(Clone, Copy, Default)]
pub(crate) struct CandidateInputCounts {
    pub(crate) entity_reads: usize,
    pub(crate) relation_reads: usize,
    pub(crate) entity_aspect_reads: usize,
    pub(crate) relation_aspect_reads: usize,
    pub(crate) adjacency_gathers: usize,
    pub(crate) adjacency_count_reads: usize,
    pub(crate) adjacency_relation_ids: usize,
    pub(crate) touched_entity_gathers: usize,
    pub(crate) touched_relation_gathers: usize,
    pub(crate) touched_partition_gathers: usize,
    pub(crate) touched_entity_slot_gathers: usize,
    pub(crate) touched_relation_slot_gathers: usize,
    pub(crate) reuse_hits: usize,
}
