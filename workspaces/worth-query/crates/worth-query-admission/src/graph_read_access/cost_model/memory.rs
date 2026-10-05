use std::fmt::{self, Write};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryGraphReadMemoryByteEstimate {
    adjacency_bytes: usize,
    reverse_adjacency_bytes: usize,
    frontier_bytes: usize,
    visited_bytes: usize,
    dedup_bytes: usize,
    predicate_bytes: usize,
    ordering_bytes: usize,
    proof_bytes: usize,
    result_bytes: usize,
}

impl WorthQueryGraphReadMemoryByteEstimate {
    pub(crate) fn empty() -> Self {
        Self {
            adjacency_bytes: 0,
            reverse_adjacency_bytes: 0,
            frontier_bytes: 0,
            visited_bytes: 0,
            dedup_bytes: 0,
            predicate_bytes: 0,
            ordering_bytes: 0,
            proof_bytes: 0,
            result_bytes: 0,
        }
    }

    pub fn adjacency_bytes(&self) -> usize {
        self.adjacency_bytes
    }

    pub fn reverse_adjacency_bytes(&self) -> usize {
        self.reverse_adjacency_bytes
    }

    pub fn frontier_bytes(&self) -> usize {
        self.frontier_bytes
    }

    pub fn visited_bytes(&self) -> usize {
        self.visited_bytes
    }

    pub fn dedup_bytes(&self) -> usize {
        self.dedup_bytes
    }

    pub fn predicate_bytes(&self) -> usize {
        self.predicate_bytes
    }

    pub fn ordering_bytes(&self) -> usize {
        self.ordering_bytes
    }

    pub fn proof_bytes(&self) -> usize {
        self.proof_bytes
    }

    pub fn result_bytes(&self) -> usize {
        self.result_bytes
    }

    pub fn index_bytes(&self) -> usize {
        self.adjacency_bytes
            + self.reverse_adjacency_bytes
            + self.frontier_bytes
            + self.visited_bytes
            + self.dedup_bytes
            + self.predicate_bytes
            + self.ordering_bytes
    }

    pub fn total_bytes(&self) -> usize {
        self.index_bytes() + self.proof_bytes + self.result_bytes
    }

    pub(crate) fn add_adjacency_bytes(&mut self, bytes: usize) {
        self.adjacency_bytes += bytes;
    }

    pub(crate) fn add_reverse_adjacency_bytes(&mut self, bytes: usize) {
        self.reverse_adjacency_bytes += bytes;
    }

    pub(crate) fn add_frontier_bytes(&mut self, bytes: usize) {
        self.frontier_bytes += bytes;
    }

    pub(crate) fn add_visited_bytes(&mut self, bytes: usize) {
        self.visited_bytes += bytes;
    }

    pub(crate) fn add_dedup_bytes(&mut self, bytes: usize) {
        self.dedup_bytes += bytes;
    }

    pub(crate) fn add_predicate_bytes(&mut self, bytes: usize) {
        self.predicate_bytes += bytes;
    }

    pub(crate) fn add_ordering_bytes(&mut self, bytes: usize) {
        self.ordering_bytes += bytes;
    }

    pub(crate) fn add_proof_bytes(&mut self, bytes: usize) {
        self.proof_bytes += bytes;
    }

    pub(crate) fn add_result_bytes(&mut self, bytes: usize) {
        self.result_bytes += bytes;
    }

    pub(crate) fn write_digest_part(&self, output: &mut dyn Write) -> fmt::Result {
        write!(
            output,
            "memory:adjacency:{}:reverse:{}:frontier:{}:visited:{}:dedup:{}:predicate:{}:ordering:{}:proof:{}:result:{}",
            self.adjacency_bytes,
            self.reverse_adjacency_bytes,
            self.frontier_bytes,
            self.visited_bytes,
            self.dedup_bytes,
            self.predicate_bytes,
            self.ordering_bytes,
            self.proof_bytes,
            self.result_bytes
        )
    }
}
