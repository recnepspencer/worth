#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ApplicationRelationCardinality {
    pub source_min: Option<u64>,
    pub source_max: Option<u64>,
    pub target_min: Option<u64>,
    pub target_max: Option<u64>,
    pub pair_min: Option<u64>,
    pub pair_max: Option<u64>,
}

impl ApplicationRelationCardinality {
    pub const fn new(
        source_min: Option<u64>,
        source_max: Option<u64>,
        target_min: Option<u64>,
        target_max: Option<u64>,
        pair_min: Option<u64>,
        pair_max: Option<u64>,
    ) -> Self {
        Self {
            source_min,
            source_max,
            target_min,
            target_max,
            pair_min,
            pair_max,
        }
    }

    pub const fn unbounded() -> Self {
        Self::new(None, None, None, None, None, None)
    }

    pub const fn has_bound(self) -> bool {
        self.source_min.is_some()
            || self.source_max.is_some()
            || self.target_min.is_some()
            || self.target_max.is_some()
            || self.pair_min.is_some()
            || self.pair_max.is_some()
    }

    pub(crate) fn is_valid(self) -> bool {
        [self.source_min, self.target_min, self.pair_min]
            .into_iter()
            .flatten()
            .all(|minimum| minimum > 0)
            && self.pair_max != Some(0)
            && [
                (self.source_min, self.source_max),
                (self.target_min, self.target_max),
                (self.pair_min, self.pair_max),
            ]
            .into_iter()
            .all(|(minimum, maximum)| match (minimum, maximum) {
                (Some(minimum), Some(maximum)) => minimum <= maximum,
                _ => true,
            })
    }
}
