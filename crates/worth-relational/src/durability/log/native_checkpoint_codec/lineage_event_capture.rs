//! Append event payload pages once; node-range routes share their member.
use super::{
    manifest_capture::corrupt,
    member_frame::{self, MemberKind},
    CapturedManifest,
};
use crate::durability::data::*;

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

    /// A live suffix may already be published while its runtime still owns its
    /// rows directly. Export those exact resident bytes without inventing a cold
    /// cell or asking an older directory's cell to serve an unknown member.
    pub(crate) fn materialize_resident_event_members(
        &mut self,
        resident: &std::collections::BTreeMap<u64, RelationalCheckpointLineageEventRow>,
        policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<(), RelationalNativeCheckpointCaptureDenial> {
        for page in &self.manifest.lineage_events {
            if !matches!(
                self.members.get(&page.member),
                Some(RelationalCheckpointMember::Carried { .. })
            ) {
                continue;
            }
            policy.check_live()?;
            let rows = page
                .event_ids
                .iter()
                .map(|id| resident.get(id).cloned())
                .collect::<Option<Vec<_>>>();
            if let Some(rows) = rows {
                let encoded =
                    member_frame::encode(MemberKind::LineageEvents, &rows).map_err(|cause| {
                        crate::storage::data::RelationalStorageUnavailable {
                            subject: crate::storage::data::RelationalStorageSubject::Lineage {
                                first: crate::identity::data::LineageId(page.first_node),
                                last: crate::identity::data::LineageId(page.last_node),
                            },
                            cause,
                        }
                    })?;
                if encoded.0 == page.member {
                    self.members.insert(
                        page.member,
                        RelationalCheckpointMember::New {
                            locator: page.member,
                            bytes: encoded.1.into(),
                        },
                    );
                    continue;
                }
            }
            if !self.guarded_members.contains_key(&page.member) {
                return Err(crate::storage::data::RelationalStorageUnavailable {
                    subject: crate::storage::data::RelationalStorageSubject::Lineage {
                        first: crate::identity::data::LineageId(page.first_node),
                        last: crate::identity::data::LineageId(page.last_node),
                    },
                    cause:
                        crate::storage::data::RelationalStorageUnavailableCause::SourceUnavailable,
                }
                .into());
            }
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
            return Err(crate::storage::data::RelationalStorageUnavailable {
                subject: crate::storage::data::RelationalStorageSubject::Lineage {
                    first: crate::identity::data::LineageId(descriptor.first_node),
                    last: crate::identity::data::LineageId(descriptor.last_node),
                },
                cause: crate::storage::data::RelationalStorageUnavailableCause::LimitExceeded {
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
                bytes: bytes.into(),
            });
        self.manifest.lineage_events.push(descriptor);
        Ok(())
    }
}
