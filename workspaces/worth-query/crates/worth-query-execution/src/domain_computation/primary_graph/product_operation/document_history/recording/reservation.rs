//! One attempt lookup and reservation for every kind of recorded step.
use super::super::{
    journal::AttemptLookup, selection::HistorySelection, WorthQueryDocumentHistoryDenial as Denial,
    WorthQueryHistoryStep,
};
use super::WorthQueryRecordingKind;
use crate::domain_computation::primary_graph::WorthQueryApplicationIdempotencyBinding;

/// A fresh attempt's reservation, or the step an earlier attempt with the same
/// meaning already recorded.
pub(in super::super) enum Attempt<T> {
    Fresh(T),
    Recorded(WorthQueryHistoryStep),
}

impl HistorySelection {
    /// Look the attempt up and check the head in `kind`'s order, then either
    /// return the step already recorded or run `reserve`. Nothing is reserved
    /// when any check refuses.
    pub(in super::super) fn lookup_and_reserve<T>(
        &mut self,
        kind: WorthQueryRecordingKind,
        meaning: WorthQueryApplicationIdempotencyBinding,
        head_is_current: impl FnOnce(&Self) -> bool,
        reserve: impl FnOnce(&mut Self) -> Result<T, Denial>,
    ) -> Result<Attempt<T>, Denial> {
        let recorded = admit_attempt(
            kind,
            || head_is_current(self),
            || match self.journal.lookup(kind.namespace(), meaning) {
                AttemptLookup::Unseen => Ok(None),
                AttemptLookup::Completed(node) => Ok(Some(node)),
                AttemptLookup::Drift => Err(Denial::IntentDrift),
                AttemptLookup::Reserved => Err(Denial::AttemptInFlight),
            },
        )?;
        match recorded {
            Some(node) => Ok(Attempt::Recorded(
                self.journal
                    .step(node)
                    .expect("a completed attempt binds a completed node")
                    .clone(),
            )),
            None => reserve(self).map(Attempt::Fresh),
        }
    }
}

/// The order of the head check and the attempt lookup. `Ok(Some(node))`
/// replays the step `node`; `Ok(None)` lets the attempt reserve. The lookup
/// does not run when an edit's head check refuses.
fn admit_attempt<N>(
    kind: WorthQueryRecordingKind,
    head_is_current: impl FnOnce() -> bool,
    recorded: impl FnOnce() -> Result<Option<N>, Denial>,
) -> Result<Option<N>, Denial> {
    let head = |current: bool| {
        if current {
            Ok(())
        } else {
            Err(Denial::HeadDiverged)
        }
    };
    if kind.head_before_lookup() {
        head(head_is_current())?;
        recorded()
    } else {
        match recorded()? {
            Some(node) => Ok(Some(node)),
            None => head(head_is_current()).map(|()| None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A retry whose attempt was recorded, on a head that has since moved.
    fn moved_and_recorded(kind: WorthQueryRecordingKind) -> Result<Option<u8>, Denial> {
        admit_attempt(kind, || false, || Ok(Some(7)))
    }

    #[test]
    fn head_check_order_is_per_kind() {
        assert!(
            matches!(
                moved_and_recorded(WorthQueryRecordingKind::Edit),
                Err(Denial::HeadDiverged)
            ),
            "an edit refuses a moved head before replaying"
        );
        for kind in [
            WorthQueryRecordingKind::Envelope,
            WorthQueryRecordingKind::Group,
        ] {
            assert!(
                matches!(moved_and_recorded(kind), Ok(Some(7))),
                "{kind:?} replays its recorded attempt first"
            );
            assert!(
                matches!(
                    admit_attempt::<u8>(kind, || false, || Ok(None)),
                    Err(Denial::HeadDiverged)
                ),
                "{kind:?} refuses a moved head before a fresh reservation"
            );
        }
        // An edit on a moved head never reads the journal.
        let refused =
            admit_attempt::<u8>(WorthQueryRecordingKind::Edit, || false, || unreachable!());
        assert!(matches!(refused, Err(Denial::HeadDiverged)));
    }
}
