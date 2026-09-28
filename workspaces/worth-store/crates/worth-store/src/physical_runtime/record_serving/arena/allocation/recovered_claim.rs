use super::*;

impl ExtentArenaAllocationOwner {
    /// Restores one authenticated, unresolved WAL obligation before allocations
    /// are exposed. Every admission check precedes changes to either index.
    pub(in crate::physical_runtime::record_serving) fn restore_claim(
        &mut self,
        range: ExtentArenaRange,
    ) -> Result<u64, ArenaAllocationDenial> {
        self.validate_range(range)?;
        let token = self.next_reservation;
        let next_token = token
            .checked_add(1)
            .ok_or(ArenaAllocationDenial::Capacity)?;
        let (new_entries, next_arena) = if range.arena().get() < self.next_arena {
            let enclosing = self
                .free
                .containing(range)
                .ok_or(ArenaAllocationDenial::Overlap)?;
            let pieces = usize::from(enclosing.offset() < range.offset())
                + usize::from(range.end() < enclosing.end());
            (self.free.len() - 1 + pieces, self.next_arena)
        } else {
            let gap = usize::try_from(range.arena().get() - self.next_arena)
                .map_err(|_| ArenaAllocationDenial::RangeBudget)?;
            let entries = self
                .free
                .len()
                .checked_add(gap)
                .and_then(|count| {
                    count.checked_add(
                        usize::from(range.offset() > 0)
                            + usize::from(range.end() < self.capacity.get()),
                    )
                })
                .ok_or(ArenaAllocationDenial::RangeBudget)?;
            (
                entries,
                range
                    .arena()
                    .get()
                    .checked_add(1)
                    .ok_or(ArenaAllocationDenial::Capacity)?,
            )
        };
        if new_entries
            .checked_add(self.reservations.len())
            .and_then(|count| count.checked_add(1))
            .is_none_or(|count| count > self.maximum_ranges)
        {
            return Err(ArenaAllocationDenial::RangeBudget);
        }
        if range.arena().get() >= self.next_arena {
            for id in self.next_arena..=range.arena().get() {
                self.free.insert(
                    ExtentArenaRange::new(ExtentArenaId::new(id).unwrap(), 0, self.capacity.get())
                        .unwrap(),
                )?;
            }
        }
        self.free.take_exact(range)?;
        self.next_arena = next_arena;
        self.next_reservation = next_token;
        self.reservations.insert(token, range);
        Ok(token)
    }
}
