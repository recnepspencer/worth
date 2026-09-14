use crate::publication::patch::data::PublishedAuthoritativeRecordPatch;
use crate::publication::patch::data::RecordStructuralChange;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum DiffFragmentKind {
    Created,
    Updated,
    Deleted,
    RetainedForAudit,
    MaterializationSuspended,
    Rematerialized,
}

impl From<RecordStructuralChange> for DiffFragmentKind {
    fn from(value: RecordStructuralChange) -> Self {
        match value {
            RecordStructuralChange::Created => Self::Created,
            RecordStructuralChange::Updated => Self::Updated,
            RecordStructuralChange::Deleted => Self::Deleted,
            RecordStructuralChange::RetainedForAudit => Self::RetainedForAudit,
            RecordStructuralChange::MaterializationSuspended => Self::MaterializationSuspended,
            RecordStructuralChange::Rematerialized => Self::Rematerialized,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DiffPreparationHeader {
    pub(crate) packet_index_floor: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DiffPreparationPacket {
    pub(crate) header: DiffPreparationHeader,
    pub(crate) authoritative_record_patches: Vec<PublishedAuthoritativeRecordPatch>,
}
