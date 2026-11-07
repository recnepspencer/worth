//! Owned resident edits preserve their level and deferred history readmission.
use super::{
    Full, Head, RelationalRegionResidency, RelationalRegionRetention, RelationalResidencyLease,
    ResidentFullRegionEdit, ResidentHistoryReadmission,
};
use crate::storage::overlay::PartitionState;

#[derive(Debug, Clone)]
pub(crate) enum ResidentRegionLease {
    Head {
        lease: RelationalResidencyLease<Head>,
        readmission: ResidentHistoryReadmission,
    },
    Full(RelationalResidencyLease<Full>),
}
impl ResidentRegionLease {
    pub(crate) fn admitted_head(
        lease: RelationalResidencyLease<Head>,
    ) -> Result<Self, crate::storage::data::RelationalStorageUnavailable> {
        let readmission = lease.cell().current_readmission().ok_or(
            crate::storage::data::RelationalStorageUnavailable {
                subject: crate::storage::data::RelationalStorageSubject::Region(
                    lease.head_view().partition_id(),
                ),
                cause: crate::storage::data::RelationalStorageUnavailableCause::SourceUnavailable,
            },
        )?;
        Ok(Self::Head { lease, readmission })
    }
    pub(crate) fn head_lease(&self) -> RelationalResidencyLease<Head> {
        match self {
            Self::Head { lease, .. } => lease.clone(),
            Self::Full(lease) => lease.as_head(),
        }
    }
    pub(crate) fn retention(&self) -> RelationalRegionRetention {
        match self {
            Self::Head { lease, .. } => RelationalRegionRetention::from_head_lease(lease.clone()),
            Self::Full(lease) => RelationalRegionRetention::from_lease(lease.clone()),
        }
    }
    pub(crate) fn into_edit(
        self,
        readmission: ResidentHistoryReadmission,
    ) -> (ResidentRegionEdit, bool) {
        let copied = match &self {
            Self::Head { lease, .. } => std::sync::Arc::strong_count(lease.cell()) > 1,
            Self::Full(lease) => lease.has_other_observers(),
        };
        let state = match self {
            Self::Full(lease) => {
                let region = lease.resident_region();
                let history = region.history_presence.basis(&region.partition);
                EditState::Current {
                    partition: lease.into_owned_partition(),
                    history,
                    readmission,
                }
            }
            Self::Head { lease, .. } => {
                let region = lease.resident_region();
                let history = region.history_presence.basis(&region.partition);
                EditState::Current {
                    partition: lease.into_owned_contents(),
                    history,
                    readmission,
                }
            }
        };
        (ResidentRegionEdit { state }, copied)
    }
    pub(crate) fn into_metadata_edit(self) -> (ResidentRegionEdit, bool) {
        match self {
            Self::Head { lease, readmission } => Self::Head {
                lease,
                readmission: readmission.clone(),
            }
            .into_edit(readmission),
            Self::Full(lease) => {
                let copied = lease.has_other_observers();
                (ResidentRegionEdit::from_full(lease), copied)
            }
        }
    }
}

pub(crate) struct ResidentRegionEdit {
    state: EditState,
}
enum EditState {
    Full(ResidentFullRegionEdit),
    Current {
        partition: PartitionState,
        history: super::history_presence::HistoryPresenceBasis,
        readmission: ResidentHistoryReadmission,
    },
}
impl ResidentRegionEdit {
    pub(crate) fn from_full(lease: RelationalResidencyLease<Full>) -> Self {
        Self {
            state: EditState::Full(lease.into_edit()),
        }
    }
    pub(crate) fn full_partition_mut(&mut self) -> Option<&mut PartitionState> {
        match &mut self.state {
            EditState::Full(edit) => Some(edit.partition_mut()),
            EditState::Current { .. } => None,
        }
    }
    pub(crate) fn partition(&self) -> &PartitionState {
        match &self.state {
            EditState::Full(edit) => edit.partition(),
            EditState::Current { partition, .. } => partition,
        }
    }
    pub(crate) fn partition_mut(&mut self) -> &mut PartitionState {
        match &mut self.state {
            EditState::Full(edit) => edit.partition_mut(),
            EditState::Current { partition, .. } => partition,
        }
    }
    pub(crate) fn transform(
        self,
        readmission: ResidentHistoryReadmission,
        transform: impl FnOnce(PartitionState) -> PartitionState,
    ) -> Self {
        let state = match self.state {
            EditState::Full(edit) => {
                // A publication may follow an earlier metadata edit in this
                // writer and import prefixes from another Head region.
                let history =
                    super::history_presence::HistoryPresence::complete().basis(edit.partition());
                EditState::Current {
                    partition: transform(edit.into_partition()),
                    history,
                    readmission,
                }
            }
            EditState::Current {
                partition,
                history,
                readmission: _,
            } => EditState::Current {
                partition: transform(partition),
                history,
                readmission,
            },
        };
        Self { state }
    }
    pub(crate) fn freeze(self) -> ResidentRegionLease {
        match self.state {
            EditState::Full(edit) => ResidentRegionLease::Full(edit.freeze()),
            EditState::Current {
                partition,
                history,
                readmission,
            } => {
                let history_presence = history.updated(&partition);
                if history_presence.is_complete() {
                    ResidentRegionLease::Full(RelationalRegionResidency::resident_uncommitted(
                        partition,
                    ))
                } else {
                    ResidentRegionLease::Head {
                        lease: RelationalRegionResidency::resident_current(
                            partition,
                            history_presence,
                            readmission.clone(),
                        ),
                        readmission,
                    }
                }
            }
        }
    }
}
