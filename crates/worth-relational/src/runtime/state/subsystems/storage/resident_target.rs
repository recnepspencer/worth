//! Stable resident custody for publication, independent of reader editions.
use std::sync::{Arc, Mutex};

use crate::identity::data::PartitionId;
use crate::storage::residency::{Full, RelationalResidencyLease, ResidentFullRegionEdit};

#[derive(Debug, Clone)]
pub(crate) struct ResidentMirrorTarget {
    id: PartitionId,
    contents: Arc<Mutex<ResidentTargetContents>>,
}

#[derive(Debug)]
enum ResidentTargetContents {
    Full {
        lease: RelationalResidencyLease<Full>,
        clear_pins: bool,
    },
    /// Only the exclusive mirror writer can hold an edit. No reader or
    /// publisher can acquire that writer until its edits have been restored.
    Editing,
}

impl ResidentMirrorTarget {
    pub(super) fn new(lease: RelationalResidencyLease<Full>, clear_pins: bool) -> Self {
        Self {
            id: lease.full_view().partition().partition_id,
            contents: Arc::new(Mutex::new(ResidentTargetContents::Full {
                lease,
                clear_pins,
            })),
        }
    }

    pub(crate) fn partition_id(&self) -> PartitionId {
        self.id
    }

    pub(super) fn same_target(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.contents, &other.contents)
    }

    pub(super) fn resident_lease(&self) -> RelationalResidencyLease<Full> {
        match &*self
            .contents
            .lock()
            .unwrap_or_else(|error| error.into_inner())
        {
            ResidentTargetContents::Full { lease, .. } => lease.clone(),
            ResidentTargetContents::Editing => unreachable!("the exclusive writer owns this edit"),
        }
    }

    pub(super) fn begin_edit(&self) -> (ResidentFullRegionEdit, bool) {
        let mut contents = self
            .contents
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        match std::mem::replace(&mut *contents, ResidentTargetContents::Editing) {
            ResidentTargetContents::Full { lease, clear_pins } => {
                let copied = lease.has_other_observers();
                let mut edit = lease.into_edit();
                if clear_pins {
                    edit.partition_mut().clear_runtime_pin_counters();
                }
                (edit, copied)
            }
            ResidentTargetContents::Editing => {
                unreachable!("the exclusive writer owns every outstanding target edit")
            }
        }
    }

    pub(super) fn restore(&self, lease: RelationalResidencyLease<Full>) {
        *self
            .contents
            .lock()
            .unwrap_or_else(|error| error.into_inner()) = ResidentTargetContents::Full {
            lease,
            clear_pins: false,
        };
    }
}
