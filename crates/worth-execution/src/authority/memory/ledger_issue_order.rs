//! A ledger's issue position supplies a total lock order across shared lineages.
use std::{cmp::Ordering, sync::Mutex};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct LedgerIssueOrder(Digits);

#[derive(Clone, Debug, Eq, PartialEq)]
enum Digits {
    Small(u64),
    Large(Vec<u64>),
}

static NEXT: Mutex<LedgerIssueOrder> = Mutex::new(LedgerIssueOrder(Digits::Small(0)));

impl LedgerIssueOrder {
    pub(super) fn issue() -> Self {
        let mut next = NEXT
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let issued = next.clone();
        next.advance();
        issued
    }

    // The ordinary position is inline. Carry extends its exact representation,
    // so an infallible budget constructor never wraps or reissues a position.
    fn advance(&mut self) {
        match &mut self.0 {
            Digits::Small(n) => {
                if let Some(next) = n.checked_add(1) {
                    *n = next;
                } else {
                    self.0 = Digits::Large(vec![1, 0]);
                }
            }
            Digits::Large(words) => {
                for word in words.iter_mut().rev() {
                    if let Some(next) = word.checked_add(1) {
                        *word = next;
                        return;
                    }
                    *word = 0;
                }
                words.insert(0, 1);
            }
        }
    }
}

impl Ord for LedgerIssueOrder {
    fn cmp(&self, other: &Self) -> Ordering {
        match (&self.0, &other.0) {
            (Digits::Small(a), Digits::Small(b)) => a.cmp(b),
            (Digits::Small(_), Digits::Large(_)) => Ordering::Less,
            (Digits::Large(_), Digits::Small(_)) => Ordering::Greater,
            (Digits::Large(a), Digits::Large(b)) => a.len().cmp(&b.len()).then_with(|| a.cmp(b)),
        }
    }
}

impl PartialOrd for LedgerIssueOrder {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn issue_positions_carry_without_wrapping_or_tying() {
        let mut last = LedgerIssueOrder(Digits::Small(u64::MAX));
        let prior = last.clone();
        last.advance();
        assert!(prior < last);
        assert_eq!(last, LedgerIssueOrder(Digits::Large(vec![1, 0])));
        let mut carried = LedgerIssueOrder(Digits::Large(vec![u64::MAX, u64::MAX]));
        let prior = carried.clone();
        carried.advance();
        assert!(prior < carried);
        assert_eq!(carried, LedgerIssueOrder(Digits::Large(vec![1, 0, 0])));
    }
}
