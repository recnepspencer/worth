use super::super::WorthQueryInvariantProjectionWork;

/// The provider version an ordinary read observed.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct WorthQueryOrdinaryReadVersion(u64);

impl WorthQueryOrdinaryReadVersion {
    pub(super) const fn from_provider_version(version: u64) -> Self {
        Self(version)
    }

    pub const fn get(self) -> u64 {
        self.0
    }
}

/// What an ordinary read closure returns: its output, how many results it
/// holds, and whether it was truncated.
///
/// Build with `complete` or `truncated`.
pub struct WorthQueryOrdinaryReadBatch<Output> {
    output: Output,
    result_count: usize,
    truncated: bool,
}

impl<Output> WorthQueryOrdinaryReadBatch<Output> {
    pub const fn complete(output: Output, result_count: usize) -> Self {
        Self {
            output,
            result_count,
            truncated: false,
        }
    }

    pub const fn truncated(output: Output, result_count: usize) -> Self {
        Self {
            output,
            result_count,
            truncated: true,
        }
    }

    pub(super) fn into_parts(self) -> (Output, usize, bool) {
        (self.output, self.result_count, self.truncated)
    }
}

/// Evidence about one ordinary read: the version read, the work spent, the
/// result count, and whether the result was truncated.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryOrdinaryReadMetadata {
    version: WorthQueryOrdinaryReadVersion,
    work: WorthQueryInvariantProjectionWork,
    result_count: usize,
    truncated: bool,
}

impl WorthQueryOrdinaryReadMetadata {
    pub const fn version(self) -> WorthQueryOrdinaryReadVersion {
        self.version
    }

    pub const fn work(self) -> WorthQueryInvariantProjectionWork {
        self.work
    }

    pub const fn result_count(self) -> usize {
        self.result_count
    }

    pub const fn truncated(self) -> bool {
        self.truncated
    }
}

/// The result of `read_admitted_operation`: the closure's output and the read's
/// metadata. No snapshot is retained.
pub struct WorthQueryOrdinaryReadProjection<Output> {
    output: Output,
    metadata: WorthQueryOrdinaryReadMetadata,
}

impl<Output> WorthQueryOrdinaryReadProjection<Output> {
    pub(super) const fn new(output: Output, metadata: WorthQueryOrdinaryReadMetadata) -> Self {
        Self { output, metadata }
    }

    pub const fn output(&self) -> &Output {
        &self.output
    }

    pub const fn metadata(&self) -> WorthQueryOrdinaryReadMetadata {
        self.metadata
    }

    pub fn into_output(self) -> Output {
        self.output
    }

    pub fn into_parts(self) -> (Output, WorthQueryOrdinaryReadMetadata) {
        (self.output, self.metadata)
    }
}

pub(super) const fn metadata(
    version: WorthQueryOrdinaryReadVersion,
    work: WorthQueryInvariantProjectionWork,
    result_count: usize,
    truncated: bool,
) -> WorthQueryOrdinaryReadMetadata {
    WorthQueryOrdinaryReadMetadata {
        version,
        work,
        result_count,
        truncated,
    }
}
