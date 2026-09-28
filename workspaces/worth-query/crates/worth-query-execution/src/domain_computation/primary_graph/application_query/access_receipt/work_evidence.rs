/// Work units one query read spent, by phase (predicate, adjacency, ordering,
/// continuation seek, projection), plus the total charged against the work
/// limit.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryApplicationQueryWorkEvidence {
    predicate_work_units: usize,
    adjacency_work_units: usize,
    ordering_work_units: usize,
    continuation_seek_work_units: usize,
    projection_work_units: usize,
    total_work_units: usize,
}

impl WorthQueryApplicationQueryWorkEvidence {
    pub(super) const fn new(
        predicate_work_units: usize,
        adjacency_work_units: usize,
        ordering_work_units: usize,
        continuation_seek_work_units: usize,
        projection_work_units: usize,
        total_work_units: usize,
    ) -> Self {
        Self {
            predicate_work_units,
            adjacency_work_units,
            ordering_work_units,
            continuation_seek_work_units,
            projection_work_units,
            total_work_units,
        }
    }

    pub const fn predicate_work_units(self) -> usize {
        self.predicate_work_units
    }

    pub const fn adjacency_work_units(self) -> usize {
        self.adjacency_work_units
    }

    pub const fn ordering_work_units(self) -> usize {
        self.ordering_work_units
    }

    pub const fn continuation_seek_work_units(self) -> usize {
        self.continuation_seek_work_units
    }

    pub const fn projection_work_units(self) -> usize {
        self.projection_work_units
    }

    pub const fn total_work_units(self) -> usize {
        self.total_work_units
    }
}
