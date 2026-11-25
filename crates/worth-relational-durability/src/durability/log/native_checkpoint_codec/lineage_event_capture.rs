//! Append event payload pages once; node-range routes share their member.
use super::manifest_capture::corrupt;
use super::member_frame;
use super::CapturedManifest;
use crate::durability::data::*;
use worth_relational_contracts::internal_port as contracts;
use worth_relational_storage::internal_port::storage::data::member_frame::MemberKind;

impl CapturedManifest {
    pub(crate) fn append_lineage_events(
        &mut self,
        mut rows: Vec<RelationalCheckpointLineageEventRow>,
        policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<(), RelationalNativeCheckpointCaptureDenial> {
        rows.sort_by_key(|row| row.event.event_id());
        if rows
            .windows(2)
            .any(|pair| pair[0].event.event_id() == pair[1].event.event_id())
        {
            return Err(corrupt("captured lineage event identity is duplicated").into());
        }
        for page in rows.chunks(LINEAGE_EVENT_PAGE_ROWS) {
            self.append_event_page(page, policy)?;
        }
        Ok(())
    }

    fn append_event_page(
        &mut self,
        rows: &[RelationalCheckpointLineageEventRow],
        policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<(), RelationalNativeCheckpointCaptureDenial> {
        policy.check_live()?;
        let (member, bytes) = member_frame::encode(MemberKind::LineageEvents, &rows)
            .map_err(|cause| corrupt(format!("lineage event encoding: {cause:?}")))?;
        if member.length > LINEAGE_EVENT_PAGE_BYTES && rows.len() > 1 {
            let (left, right) = rows.split_at(rows.len() / 2);
            self.append_event_page(left, policy)?;
            return self.append_event_page(right, policy);
        }
        let descriptor = RelationalCheckpointLineageEventPage::from_rows(member, rows)
            .ok_or_else(|| corrupt("invalid captured lineage event page"))?;
        if member.length > LINEAGE_EVENT_PAGE_BYTES {
            return Err(contracts::storage::data::storage_unavailable::RelationalStorageUnavailable {
                subject: contracts::storage::data::storage_unavailable::RelationalStorageSubject::Lineage {
                    first: contracts::identity::data::primitives::LineageId(descriptor.first_node),
                    last: contracts::identity::data::primitives::LineageId(descriptor.last_node),
                },
                cause: contracts::storage::data::storage_unavailable::RelationalStorageUnavailableCause::LimitExceeded {
                    length: member.length,
                    limit: LINEAGE_EVENT_PAGE_BYTES,
                },
            }
            .into());
        }
        self.members
            .entry(member)
            .or_insert(RelationalCheckpointMember::New {
                locator: member,
                bytes: worth_execution::ExecutionImmutableBytes::from_external_bytes(
                    std::sync::Arc::new(bytes.into_boxed_slice()),
                ),
            });
        let mut rows = self.manifest.lineage_events.ordinary_owned_copy();
        rows.push(descriptor);
        self.manifest.lineage_events = CheckpointLineageEventPages::from_ordinary(rows);
        Ok(())
    }
}
