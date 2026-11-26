//! Where a typed live subscription keeps its open native subscription:
//! borrowed beside its Home, or held by an owned subscription that the Home
//! releases (closes) when it closes.
use std::cell::{RefCell, RefMut};
use std::rc::Rc;

use worth_query_declaration::facade::{
    application_query::ApplicationQueryIntent, application_schema::ApplicationSchema,
};
use worth_query_execution::facade::primary_graph::{
    WorthQueryApplicationProjection, WorthQueryCommittedProductPublication,
};
use worth_query_publication::facade::application_entry::WorthQueryApplicationWholeResultLiveSubscription as Native;

use super::{QueryMarker, QueryRow};
use crate::application_home::owned_holds::{
    LocalHoldRegistration, OwnedRuntimeHold, OwnedRuntimeHolds,
};

pub(super) enum LiveSlot<'home, Schema, Intent>
where
    Schema: ApplicationSchema,
    Intent: ApplicationQueryIntent<Schema>,
    QueryRow<Schema, Intent>: WorthQueryApplicationProjection<Schema, QueryMarker<Schema, Intent>>,
{
    Direct(Native<'home, Schema, Intent>),
    Held(Rc<HeldLive<Schema, Intent>>),
}

/// An owned subscription's native subscription, `None` once released.
pub(super) struct HeldLive<Schema, Intent>
where
    Schema: ApplicationSchema,
    Intent: ApplicationQueryIntent<Schema>,
    QueryRow<Schema, Intent>: WorthQueryApplicationProjection<Schema, QueryMarker<Schema, Intent>>,
{
    native: RefCell<Option<Native<'static, Schema, Intent>>>,
    /// The latest delivered publication; its receipt pins the Home's runtime.
    publication: RefCell<Option<WorthQueryCommittedProductPublication>>,
    _registration: LocalHoldRegistration,
}

impl<Schema, Intent> HeldLive<Schema, Intent>
where
    Schema: ApplicationSchema,
    Intent: ApplicationQueryIntent<Schema>,
    QueryRow<Schema, Intent>: WorthQueryApplicationProjection<Schema, QueryMarker<Schema, Intent>>,
{
    /// Holds `native` and registers it with its Home's `holds` on this thread.
    pub(super) fn new(
        native: Native<'static, Schema, Intent>,
        holds: &OwnedRuntimeHolds,
    ) -> Rc<Self>
    where
        Intent: 'static,
    {
        Rc::new_cyclic(|held| {
            let hold: std::rc::Weak<dyn OwnedRuntimeHold> = held.clone();
            Self {
                native: RefCell::new(Some(native)),
                publication: RefCell::new(None),
                _registration: holds.register_local(hold),
            }
        })
    }

    pub(super) fn native(&self) -> RefMut<'_, Option<Native<'static, Schema, Intent>>> {
        self.native.borrow_mut()
    }

    pub(super) fn publication(&self) -> RefMut<'_, Option<WorthQueryCommittedProductPublication>> {
        self.publication.borrow_mut()
    }
}

impl<Schema, Intent> OwnedRuntimeHold for HeldLive<Schema, Intent>
where
    Schema: ApplicationSchema,
    Intent: ApplicationQueryIntent<Schema>,
    QueryRow<Schema, Intent>: WorthQueryApplicationProjection<Schema, QueryMarker<Schema, Intent>>,
{
    /// Closes the subscription and drops its publication receipt, so neither
    /// holds the Home's runtime.
    fn release(&self) {
        if let Ok(mut publication) = self.publication.try_borrow_mut() {
            publication.take();
        }
        let native = match self.native.try_borrow_mut() {
            Ok(mut native) => native.take(),
            Err(_) => None,
        };
        if let Some(native) = native {
            let _ = native.close();
        }
    }
}
