//! Scoped observations of this installation's actual borrowed update receiver.
use super::*;
use std::{cell::RefCell, marker::PhantomData, rc::Rc, sync::Arc};

type RowChanges = Vec<(query::ItemRow, query::ItemRow)>;
type Observer = Box<dyn FnMut(Vec<query::ItemRow>, Vec<query::ItemRow>, RowChanges) + Send>;
struct Recording {
    receiver: Arc<Calls>,
    observe: Observer,
}
thread_local! {
    static CURRENT: RefCell<Option<Recording>> = const { RefCell::new(None) };
}

pub(in super::super) struct SourceUpdateRecording {
    previous: Option<Recording>,
    _thread: PhantomData<Rc<()>>,
}
impl Drop for SourceUpdateRecording {
    fn drop(&mut self) {
        CURRENT.with(|current| current.replace(self.previous.take()));
    }
}
pub(in super::super) fn arm(receiver: &Arc<Calls>, observe: Observer) -> SourceUpdateRecording {
    SourceUpdateRecording {
        previous: CURRENT.with(|current| {
            current.replace(Some(Recording {
                receiver: Arc::clone(receiver),
                observe,
            }))
        }),
        _thread: PhantomData,
    }
}

/// No RefCell guard or observer remains active during projection or callbacks.
struct RecordingLoan(Option<Recording>);
impl Drop for RecordingLoan {
    fn drop(&mut self) {
        CURRENT.with(|current| current.replace(self.0.take()));
    }
}
pub(super) fn record(
    calls: &Calls,
    before: &WorthQueryManagedIndexSourceRows<'_, Catalog, PrefixIndex>,
    after: &WorthQueryManagedIndexSourceRows<'_, Catalog, PrefixIndex>,
    changes: &[WorthQueryManagedIndexSourceChange],
) -> Result<(), Stop> {
    let recording = CURRENT.with(|current| {
        let mut current = current.borrow_mut();
        if current
            .as_ref()
            .is_some_and(|recording| std::ptr::eq(calls, recording.receiver.as_ref()))
        {
            current.take()
        } else {
            None
        }
    });
    let Some(recording) = recording else {
        return Ok(());
    };
    let mut recording = RecordingLoan(Some(recording));
    let changed = changes
        .iter()
        .map(|change| {
            let before = before.get(change.before().expect("existing row"))?;
            let after = after.get(change.after().expect("surviving row"))?;
            Ok((project(&before)?, project(&after)?))
        })
        .collect::<Result<_, Stop>>()?;
    (recording
        .0
        .as_mut()
        .expect("the scoped recording is retained")
        .observe)(read(before)?, read(after)?, changed);
    Ok(())
}
fn read(
    rows: &WorthQueryManagedIndexSourceRows<'_, Catalog, PrefixIndex>,
) -> Result<Vec<query::ItemRow>, Stop> {
    rows.slots()
        .map(|slot| project(&rows.get(slot?)?))
        .collect()
}
fn project(
    row: &WorthQueryManagedIndexSourceRow<'_, Catalog, PrefixIndex>,
) -> Result<query::ItemRow, Stop> {
    query::ItemRow::project(row).map_err(|_| Stop::Domain(PrefixIndexDenial::InvalidTopology))
}
