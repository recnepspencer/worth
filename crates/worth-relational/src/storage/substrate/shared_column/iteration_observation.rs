//! Passive lexical observation of actual prefix-column traversal.
//! This catches uncharged whole-region loops through the public SDK court.
use std::cell::Cell;

thread_local! { static VISITS: Cell<Option<u64>> = const { Cell::new(None) }; }

struct PrefixTraversalRecording {
    previous: Option<u64>,
    _thread: std::marker::PhantomData<std::rc::Rc<()>>,
}

/// Counts physically visited history-prefix entries on this thread, including
/// uncharged traversals. The lexical owner restores nesting and unwinding.
/// This is passive inspection and does not admit work or data.
pub fn observe_history_prefix_traversal<R>(operation: impl FnOnce() -> R) -> (R, u64) {
    let guard = PrefixTraversalRecording {
        previous: VISITS.with(|slot| slot.replace(Some(0))),
        _thread: std::marker::PhantomData,
    };
    let result = operation();
    let visits = VISITS.with(|slot| slot.get().expect("live recorder"));
    drop(guard);
    (result, visits)
}

impl Drop for PrefixTraversalRecording {
    fn drop(&mut self) {
        VISITS.with(|slot| slot.set(self.previous));
    }
}
pub(super) fn visited<T>() {
    // Other column monomorphizations never consult the observer.
    if std::any::type_name::<T>()
        != std::any::type_name::<
            crate::storage::substrate::StorageAllocation<
                Option<crate::storage::substrate::RecordHistoryPrefix>,
            >,
        >()
    {
        return;
    }
    VISITS.with(|slot| {
        if let Some(visits) = slot.get() {
            slot.set(Some(visits.saturating_add(1)));
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn traversal_recording_restores_its_predecessor_after_unwinding() {
        let values: crate::storage::substrate::SharedColumn<
            crate::storage::substrate::StorageAllocation<
                Option<crate::storage::substrate::RecordHistoryPrefix>,
            >,
        > = crate::storage::substrate::SharedColumn::from(vec![
            None::<
                crate::storage::substrate::RecordHistoryPrefix,
            >;
            2
        ]);
        let (_, visits) = observe_history_prefix_traversal(|| {
            assert!(values.iter().all(Option::is_none));
            let interrupted = std::panic::catch_unwind(|| {
                observe_history_prefix_traversal(|| panic!("stop capture"));
            });
            assert!(interrupted.is_err());
            assert!(values.iter().all(Option::is_none));
        });
        assert_eq!(visits, (values.len() * 2) as u64);
        VISITS.with(|slot| assert!(slot.get().is_none()));
    }
}
