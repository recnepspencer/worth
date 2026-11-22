use std::num::NonZeroUsize;
const DEFAULT_CONCURRENT_GRAPH_WORK: usize = 64;
/// Simultaneous native graph-work admission; not a query work/byte invoice.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryApplicationQueryResourceProfile {
    maximum_concurrent_graph_work: NonZeroUsize,
}
impl WorthQueryApplicationQueryResourceProfile {
    pub const fn new(maximum_concurrent_graph_work: NonZeroUsize) -> Self {
        Self {
            maximum_concurrent_graph_work,
        }
    }
    pub const fn with_maximum_concurrent_graph_work(mut self, maximum: NonZeroUsize) -> Self {
        self.maximum_concurrent_graph_work = maximum;
        self
    }
    pub const fn maximum_concurrent_graph_work(self) -> NonZeroUsize {
        self.maximum_concurrent_graph_work
    }
}
impl Default for WorthQueryApplicationQueryResourceProfile {
    fn default() -> Self {
        Self::new(
            NonZeroUsize::new(DEFAULT_CONCURRENT_GRAPH_WORK)
                .expect("default concurrent graph work is non-zero"),
        )
    }
}
