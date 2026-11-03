#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct WorthQueryInvariantProjectionWorkBudget {
    remaining: Option<usize>,
    exceeded: bool,
    overflowed: bool,
    consumed: usize,
    /// Start and maximum preflight demand of the current owner call.
    call: Option<(usize, usize)>,
}

impl WorthQueryInvariantProjectionWorkBudget {
    pub(super) const fn unbounded() -> Self {
        Self {
            remaining: None,
            exceeded: false,
            overflowed: false,
            consumed: 0,
            call: None,
        }
    }

    pub(super) const fn bounded(maximum: usize) -> Self {
        Self {
            remaining: Some(maximum),
            exceeded: false,
            overflowed: false,
            consumed: 0,
            call: None,
        }
    }

    pub(super) fn can_afford(&mut self, maximum_work: usize) -> bool {
        if self.exceeded {
            return false;
        }
        if self.consumed.checked_add(maximum_work).is_none() {
            self.overflowed = true;
            self.exceeded = true;
            return false;
        }
        if self
            .remaining
            .is_some_and(|remaining| remaining < maximum_work)
        {
            self.exceeded = true;
            return false;
        }
        self.record_call_demand(maximum_work);
        true
    }

    #[must_use]
    pub(super) fn consume(&mut self, actual_work: usize) -> bool {
        if !self.can_afford(actual_work) {
            return false;
        }
        self.consumed += actual_work;
        if let Some(remaining) = self.remaining.as_mut() {
            *remaining = remaining
                .checked_sub(actual_work)
                .expect("provider work was preflighted before execution");
        }
        true
    }

    fn record_call_demand(&mut self, maximum_work: usize) {
        if let Some((start, demand)) = &mut self.call {
            let needed = self
                .consumed
                .checked_sub(*start)
                .and_then(|spent| spent.checked_add(maximum_work));
            if let Some(needed) = needed {
                *demand = (*demand).max(needed);
            } else {
                // The call cannot be carried when its charge is unrepresentable.
                self.call = None;
            }
        }
    }

    /// Rechecks need the adjacency read's existing charge plus a final admission
    /// check. A read refused before charging retains the empty-read allowance.
    /// Public diagnostic observations do not determine this private demand.
    pub(super) fn adjacency_recomparison_limit_since(self, before: Self) -> usize {
        self.consumed
            .saturating_sub(before.consumed)
            .saturating_add(1)
            .max(2)
    }

    pub(super) fn begin_call(&mut self) {
        self.call = Some((self.consumed, 0));
    }

    /// Required finite allowance and actual charge, independent of whether
    /// this execution selected a work ceiling.
    pub(super) fn end_call(&mut self) -> Option<(usize, usize)> {
        let (start, demand) = self.call.take()?;
        Some((demand, self.consumed.checked_sub(start)?))
    }

    /// Charges a call an earlier run made over the same facts, as if it were
    /// made now: it passes exactly when `demand` remains, and then consumes
    /// what it consumed through the budget's one checked spend. False,
    /// charging nothing, when it would not pass.
    pub(super) fn carry(&mut self, demand: usize, consumed: usize) -> bool {
        if self.exceeded
            || self.remaining.is_some_and(|remaining| remaining < demand)
            || consumed > demand
        {
            return false;
        }
        if self.consumed.checked_add(demand).is_none() {
            self.overflowed = true;
            self.exceeded = true;
            return false;
        }
        self.consume(consumed)
    }

    pub(super) const fn overflowed(&self) -> bool {
        self.overflowed
    }

    pub(super) fn mark_exceeded(&mut self) {
        self.exceeded = true;
    }

    pub(super) const fn remaining(&self) -> Option<usize> {
        self.remaining
    }

    pub(super) const fn exceeded(&self) -> bool {
        self.exceeded
    }
}

#[cfg(test)]
mod tests {
    use super::WorthQueryInvariantProjectionWorkBudget as Budget;

    #[test]
    fn absent_projection_work_keeps_unrepresentable_charges_owned_and_refused() {
        let mut work = Budget::unbounded();
        let before = work;
        assert!(work.consume(usize::MAX));
        assert_eq!(work.adjacency_recomparison_limit_since(before), usize::MAX);
        let before_stop = work;
        assert!(!work.consume(1));
        assert_eq!(work.adjacency_recomparison_limit_since(before_stop), 2);
        assert_eq!(work.consumed, usize::MAX);
        assert_eq!(work.remaining(), None);
        assert!(work.exceeded());
        assert!(work.overflowed());
        let mut carried = Budget::unbounded();
        assert!(carried.consume(usize::MAX));
        assert!(!carried.carry(1, 1));
        assert!(carried.overflowed());
        assert_eq!(carried.consumed, usize::MAX);
    }
}
