//! Stable resident custody for current publication and Full maintenance.
use crate::identity::data::PartitionId;
use crate::storage::residency::{
    Full, Head, RelationalRegionRetention, RelationalResidencyLease, ResidentHistoryReadmission,
    ResidentRegionEdit, ResidentRegionLease,
};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub(crate) struct ResidentMirrorTarget {
    id: PartitionId,
    contents: Arc<Mutex<ResidentTargetContents>>,
}
#[derive(Debug)]
enum ResidentTargetContents {
    Resident {
        lease: ResidentRegionLease,
        clear_pins: bool,
    },
    Editing,
}
impl ResidentMirrorTarget {
    pub(super) fn new(lease: RelationalResidencyLease<Full>, clear_pins: bool) -> Self {
        let id = lease.full_view().partition().partition_id;
        Self {
            id,
            contents: Arc::new(Mutex::new(ResidentTargetContents::Resident {
                lease: ResidentRegionLease::Full(lease),
                clear_pins,
            })),
        }
    }
    pub(super) fn from_head(
        lease: RelationalResidencyLease<Head>,
        clear_pins: bool,
    ) -> Result<Self, crate::storage::data::RelationalStorageUnavailable> {
        let id = lease.head_view().partition_id();
        Ok(Self {
            id,
            contents: Arc::new(Mutex::new(ResidentTargetContents::Resident {
                lease: ResidentRegionLease::admitted_head(lease)?,
                clear_pins,
            })),
        })
    }
    pub(crate) fn partition_id(&self) -> PartitionId {
        self.id
    }
    pub(super) fn same_target(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.contents, &other.contents)
    }
    pub(super) fn is_full(&self) -> bool {
        matches!(
            &*self
                .contents
                .lock()
                .unwrap_or_else(|error| error.into_inner()),
            ResidentTargetContents::Resident {
                lease: ResidentRegionLease::Full(_),
                ..
            }
        )
    }
    pub(super) fn retention(&self) -> RelationalRegionRetention {
        match &*self
            .contents
            .lock()
            .unwrap_or_else(|error| error.into_inner())
        {
            ResidentTargetContents::Resident { lease, .. } => lease.retention(),
            ResidentTargetContents::Editing => unreachable!(
                /* Proof: PartitionEditionWriter holds RwLockWriteGuard<CurrentMirror> for its whole lifetime. Every edit entry is reused before taking a target again, and finish_edits restores its lease before releasing that guard; mirror accessors cannot run concurrently with an edit. */
                "exclusive writer owns this edit"
            ),
        }
    }
    pub(super) fn promote_full(&self, lease: RelationalResidencyLease<Full>) {
        let mut contents = self
            .contents
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        if let ResidentTargetContents::Resident { lease: current, .. } = &mut *contents {
            *current = ResidentRegionLease::Full(lease);
        }
    }
    pub(super) fn begin_current_edit(
        &self,
        readmission: ResidentHistoryReadmission,
    ) -> (ResidentRegionEdit, bool) {
        let mut contents = self
            .contents
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        match std::mem::replace(&mut *contents, ResidentTargetContents::Editing) {
            ResidentTargetContents::Resident { lease, clear_pins } => {
                let (mut edit, copied) = lease.into_edit(readmission);
                if clear_pins {
                    edit.partition_mut().clear_runtime_pin_counters();
                }
                (edit, copied)
            }
            ResidentTargetContents::Editing => {
                unreachable!(
                    /* Proof: PartitionEditionWriter holds RwLockWriteGuard<CurrentMirror> for its whole lifetime. Every edit entry is reused before taking a target again, and finish_edits restores its lease before releasing that guard; mirror accessors cannot run concurrently with an edit. */
                    "exclusive writer owns every target edit"
                )
            }
        }
    }
    pub(super) fn current_lease(&self) -> Option<RelationalResidencyLease<Head>> {
        match &*self
            .contents
            .lock()
            .unwrap_or_else(|error| error.into_inner())
        {
            ResidentTargetContents::Resident { lease, .. } => Some(lease.head_lease()),
            ResidentTargetContents::Editing => None,
        }
    }
    pub(super) fn begin_metadata_edit(&self) -> (ResidentRegionEdit, bool) {
        let mut contents = self
            .contents
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        match std::mem::replace(&mut *contents, ResidentTargetContents::Editing) {
            ResidentTargetContents::Resident { lease, clear_pins } => {
                let (mut edit, copied) = lease.into_metadata_edit();
                if clear_pins {
                    edit.partition_mut().clear_runtime_pin_counters();
                }
                (edit, copied)
            }
            ResidentTargetContents::Editing => unreachable!(
                /* Proof: PartitionEditionWriter holds RwLockWriteGuard<CurrentMirror> for its whole lifetime. Every edit entry is reused before taking a target again, and finish_edits restores its lease before releasing that guard; mirror accessors cannot run concurrently with an edit. */
                "exclusive writer owns target edit"
            ),
        }
    }
    pub(super) fn begin_full_edit(&self) -> Option<(ResidentRegionEdit, bool)> {
        let mut contents = self
            .contents
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        if !matches!(
            &*contents,
            ResidentTargetContents::Resident {
                lease: ResidentRegionLease::Full(_),
                ..
            }
        ) {
            return None;
        }
        let ResidentTargetContents::Resident {
            lease: ResidentRegionLease::Full(lease),
            clear_pins,
        } = std::mem::replace(&mut *contents, ResidentTargetContents::Editing)
        else {
            unreachable!(
                /* Proof: The immediately preceding matches! holds this same contents Mutex and rejects Editing and Head. No mutation intervenes before mem::replace, so its removed value is Resident Full. */
                "checked resident Full custody"
            )
        };
        let copied = lease.has_other_observers();
        let mut edit = ResidentRegionEdit::from_full(lease);
        if clear_pins {
            edit.partition_mut().clear_runtime_pin_counters();
        }
        Some((edit, copied))
    }
    pub(super) fn restore(&self, lease: ResidentRegionLease) {
        *self
            .contents
            .lock()
            .unwrap_or_else(|error| error.into_inner()) = ResidentTargetContents::Resident {
            lease,
            clear_pins: false,
        };
    }
}
