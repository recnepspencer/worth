/// Physical candidate-input materializations and reuse observed by the
/// invariant engine. These counters do not confer graph read authority.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct RelationalCandidateInputCounters {
    pub entity_reads: usize,
    pub relation_reads: usize,
    pub entity_aspect_reads: usize,
    pub relation_aspect_reads: usize,
    pub adjacency_gathers: usize,
    pub reuse_hits: usize,
}

impl RelationalCandidateInputCounters {
    pub fn since(self, before: Self) -> Self {
        Self {
            entity_reads: self.entity_reads.saturating_sub(before.entity_reads),
            relation_reads: self.relation_reads.saturating_sub(before.relation_reads),
            entity_aspect_reads: self
                .entity_aspect_reads
                .saturating_sub(before.entity_aspect_reads),
            relation_aspect_reads: self
                .relation_aspect_reads
                .saturating_sub(before.relation_aspect_reads),
            adjacency_gathers: self
                .adjacency_gathers
                .saturating_sub(before.adjacency_gathers),
            reuse_hits: self.reuse_hits.saturating_sub(before.reuse_hits),
        }
    }
}
