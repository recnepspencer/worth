use worth_store_physical_format::RecordAnchor;

use super::membership::{SelectedRecord, ValueMembership};
use super::verified_value::{read_verified, PayloadRetention};
use super::{AnchoredRecord, AnchoredRecordName, AnchoredRecordReadFailure, AnchoredRecords};
use crate::physical_runtime::PhysicalRecordReader;

type Failure = AnchoredRecordReadFailure;

impl AnchoredRecords<'_> {
    /// The current value of `name` with its member list, or `None` when none
    /// was ever published.
    ///
    /// The frame, the member table and the payload are verified against the
    /// record digest the anchor carries; damage refuses and is never reported
    /// as absence. A payload larger than `maximum_payload_bytes` refuses
    /// before it is read. Members are not read; see `read_member`.
    pub fn read(
        &self,
        name: AnchoredRecordName,
        maximum_payload_bytes: u64,
    ) -> Result<Option<AnchoredRecord>, Failure> {
        let reader = self.reader()?;
        let Some(anchor) = self.current_anchor(&reader, name) else {
            return Ok(None);
        };
        let verified = read_verified(
            &reader,
            name,
            anchor,
            PayloadRetention::Retain {
                maximum_bytes: maximum_payload_bytes,
            },
        )?;
        let membership = self.remember(
            name,
            ValueMembership::new(
                SelectedRecord::of(anchor),
                verified.header,
                verified.members,
            ),
        );
        Ok(Some(AnchoredRecord {
            membership: membership.described(name),
            bytes: verified.payload.expect("a retained read keeps its payload"),
        }))
    }

    pub(super) fn reader(&self) -> Result<PhysicalRecordReader, Failure> {
        self.runtime.records().map_err(Failure::Protection)
    }

    /// The anchor `name` selects in the reader's protected root.
    pub(super) fn current_anchor(
        &self,
        reader: &PhysicalRecordReader,
        name: AnchoredRecordName,
    ) -> Option<RecordAnchor> {
        reader.selected_record_anchors().get(name.anchor())
    }
}
