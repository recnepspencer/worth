use std::collections::BTreeMap;

use crate::runtime::WorthQueryRuntimeFacadeFamily;

use super::super::support_snapshot::{WorthQuerySupportSnapshot, WorthQuerySupportSnapshotRow};
use super::error::{WorthQuerySupportPinningError, WorthQuerySupportPinningErrorKind};

#[derive(Debug)]
pub(crate) struct SupportPinSnapshotIndex<'a> {
    by_family: BTreeMap<&'a str, &'a WorthQuerySupportSnapshotRow>,
}

impl<'a> SupportPinSnapshotIndex<'a> {
    pub(crate) fn new(
        snapshot: &'a WorthQuerySupportSnapshot,
    ) -> Result<Self, WorthQuerySupportPinningError> {
        let mut by_family = BTreeMap::new();
        for row in snapshot.rows() {
            if let Some(family) = row.facade_family()
                && by_family.insert(family, row).is_some()
            {
                return Err(WorthQuerySupportPinningError::with_family(
                    WorthQuerySupportPinningErrorKind::DuplicateSnapshotFamily,
                    "support pin snapshot contains duplicate facade family rows",
                    family,
                ));
            }
        }
        Ok(Self { by_family })
    }

    pub(crate) fn required_row(
        &self,
        family: WorthQueryRuntimeFacadeFamily,
    ) -> Result<&'a WorthQuerySupportSnapshotRow, WorthQuerySupportPinningError> {
        self.by_family.get(family.as_str()).copied().ok_or_else(|| {
            WorthQuerySupportPinningError::with_family(
                WorthQuerySupportPinningErrorKind::SnapshotRowMissing,
                "support pin required row is absent from snapshot",
                family.as_str(),
            )
        })
    }

    pub(crate) fn optional_row(
        &self,
        family: WorthQueryRuntimeFacadeFamily,
    ) -> Option<&'a WorthQuerySupportSnapshotRow> {
        self.by_family.get(family.as_str()).copied()
    }
}
