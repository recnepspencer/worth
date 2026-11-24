use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StorageInconsistencyScan {
    MaxSnapshotEntities,
    HistoricalMaxSnapshotEntities,
    LiveRecordSidecar,
    HistoricalUniqueEntityAspectField,
}

impl StorageInconsistencyScan {
    pub const fn diagnostic_label(self) -> &'static str {
        match self {
            Self::MaxSnapshotEntities => "max_snapshot_entities",
            Self::HistoricalMaxSnapshotEntities => "historical_max_snapshot_entities",
            Self::LiveRecordSidecar => "live_record_sidecar",
            Self::HistoricalUniqueEntityAspectField => "historical_unique_entity_aspect_field",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StorageInconsistencyLookup {
    EntityKindInState,
}

impl StorageInconsistencyLookup {
    pub const fn diagnostic_label(self) -> &'static str {
        match self {
            Self::EntityKindInState => "entity_kind_in_state",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StorageInconsistencyFailure {
    MissingSlot,
    MissingKindId,
}

impl StorageInconsistencyFailure {
    pub const fn diagnostic_label(self) -> &'static str {
        match self {
            Self::MissingSlot => "missing_slot",
            Self::MissingKindId => "missing_kind_id",
        }
    }
}
