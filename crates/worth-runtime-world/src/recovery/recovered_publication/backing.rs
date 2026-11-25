//! Type-erased shared handles for recovered publication records and native observations.

use std::ops::Deref;
use std::sync::Arc;
use worth_proof::RelationalPublicationReadmitted;
use worth_relational::facade::durability::{
    ReadmittedRelationalAssociatedEffect, RelationalRecoveredPublicationCustody,
};

use super::RecoveredWorldPublicationRecord;

trait RecoveredWorldPublicationRecordBacking: Send + Sync + std::fmt::Debug {
    fn as_record(&self) -> &RecoveredWorldPublicationRecord;
}

impl RecoveredWorldPublicationRecordBacking for RecoveredWorldPublicationRecord {
    fn as_record(&self) -> &RecoveredWorldPublicationRecord {
        self
    }
}

#[derive(Debug, Clone)]
pub(crate) struct SharedRecoveredWorldPublicationRecord(
    Arc<dyn RecoveredWorldPublicationRecordBacking>,
);

impl SharedRecoveredWorldPublicationRecord {
    pub(crate) fn from_arc(record: Arc<RecoveredWorldPublicationRecord>) -> Self {
        Self(record)
    }
}

impl Deref for SharedRecoveredWorldPublicationRecord {
    type Target = RecoveredWorldPublicationRecord;

    fn deref(&self) -> &Self::Target {
        self.0.as_record()
    }
}

trait RecoveredNativeObservationSource: Send + Sync + std::fmt::Debug {
    fn effect(&self) -> Option<&ReadmittedRelationalAssociatedEffect>;
    fn original(
        &self,
    ) -> Option<&RelationalPublicationReadmitted<RelationalRecoveredPublicationCustody>>;
}

impl RecoveredNativeObservationSource
    for RelationalPublicationReadmitted<RelationalRecoveredPublicationCustody>
{
    fn effect(&self) -> Option<&ReadmittedRelationalAssociatedEffect> {
        Some(self.custody().effect())
    }

    fn original(
        &self,
    ) -> Option<&RelationalPublicationReadmitted<RelationalRecoveredPublicationCustody>> {
        Some(self)
    }
}

impl RecoveredNativeObservationSource for crate::history::PreparedReadmittedPublicationRecord {
    fn effect(&self) -> Option<&ReadmittedRelationalAssociatedEffect> {
        self.effect()
    }

    fn original(
        &self,
    ) -> Option<&RelationalPublicationReadmitted<RelationalRecoveredPublicationCustody>> {
        None
    }
}

#[derive(Debug, Clone)]
pub(crate) struct RecoveredNativeObservation(Arc<dyn RecoveredNativeObservationSource>);

impl RecoveredNativeObservation {
    pub(crate) fn original(
        native: Arc<RelationalPublicationReadmitted<RelationalRecoveredPublicationCustody>>,
    ) -> Self {
        Self(native)
    }

    pub(crate) fn prepared(
        record: Arc<crate::history::PreparedReadmittedPublicationRecord>,
    ) -> Self {
        Self(record)
    }

    pub(super) fn effect(&self) -> Option<&ReadmittedRelationalAssociatedEffect> {
        self.0.effect()
    }

    pub(crate) fn original_native(
        &self,
    ) -> Option<&RelationalPublicationReadmitted<RelationalRecoveredPublicationCustody>> {
        self.0.original()
    }
}
