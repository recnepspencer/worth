#[derive(Default)]
pub(super) struct ArenaRanges {
    pub(super) routed: Vec<(u64, u64)>,
    pub(super) free: Vec<(u64, u64)>,
    pub(super) historical: Vec<(u64, u64)>,
    pub(super) uncertain: Vec<(u64, u64)>,
}

impl ArenaRanges {
    pub(super) fn geometry_damage(&self, capacity: u64, alignment: u64) -> Option<(u64, u64)> {
        self.routed
            .iter()
            .chain(&self.free)
            .chain(&self.historical)
            .copied()
            .find(|&(start, end)| {
                alignment == 0
                    || start % alignment != 0
                    || end % alignment != 0
                    || end > capacity
                    || start >= end
            })
    }
    pub(super) fn inspect(&mut self, length: u64) -> Option<(u64, u64)> {
        self.routed.sort_unstable();
        self.routed.dedup();
        self.free.sort_unstable();
        for ranges in [&self.routed, &self.free] {
            for pair in ranges.windows(2) {
                if pair[0].1 > pair[1].0 {
                    return Some((pair[1].0, pair[0].1.min(pair[1].1)));
                }
            }
        }
        let mut protected = self
            .routed
            .iter()
            .chain(&self.historical)
            .copied()
            .collect::<Vec<_>>();
        protected.sort_unstable();
        let mut free_cursor = 0;
        for (start, end) in protected {
            while free_cursor < self.free.len() && self.free[free_cursor].1 <= start {
                free_cursor += 1;
            }
            if let Some(&(free_start, free_end)) = self.free.get(free_cursor) {
                if free_start < end {
                    return Some((start.max(free_start), end.min(free_end)));
                }
            }
        }
        let mut covered = self
            .routed
            .iter()
            .chain(&self.free)
            .chain(&self.historical)
            .chain(&self.uncertain)
            .copied()
            .collect::<Vec<_>>();
        covered.sort_unstable();
        let mut frontier = 0;
        for (start, end) in covered {
            if start > frontier && frontier < length {
                return Some((frontier, start.min(length)));
            }
            frontier = frontier.max(end);
        }
        (frontier < length).then_some((frontier, length))
    }
    pub(super) fn has_unproven_gap(&self, length: u64) -> bool {
        let mut known = self
            .routed
            .iter()
            .chain(&self.free)
            .chain(&self.historical)
            .copied()
            .collect::<Vec<_>>();
        known.sort_unstable();
        let mut frontier = 0;
        for (start, end) in known {
            if start > frontier && frontier < length {
                return true;
            }
            frontier = frontier.max(end);
        }
        frontier < length
    }
}
