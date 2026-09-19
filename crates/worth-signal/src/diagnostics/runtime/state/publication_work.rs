/// Actual work at the bounded diagnostic-history publication boundary.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct DiagnosticPublicationWork {
    pub(crate) recorded: u64,
    pub(crate) evicted: u64,
}

impl DiagnosticPublicationWork {
    pub(crate) fn accumulate(&mut self, published: Self) {
        self.recorded += published.recorded;
        self.evicted += published.evicted;
    }
}
