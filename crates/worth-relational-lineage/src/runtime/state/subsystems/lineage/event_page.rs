//! Request-funded readmission of one event payload and its complete range routing.
use crate::contracts::storage::data::storage_unavailable::{
    RelationalStorageSubject, RelationalStorageUnavailable,
    RelationalStorageUnavailableCause as Cause,
};
use crate::durability::data::lineage_event_pages::{
    RelationalCheckpointLineageEventPage, RelationalCheckpointLineageEventRow,
};
use serde::de::{Error, SeqAccess, Visitor};
use worth_relational_storage::internal_port::storage::data::member_frame;
use worth_relational_storage::internal_port::storage::data::member_frame::MemberKind;
use worth_relational_storage::internal_port::storage::residency::demand_failure::RelationalResidencyDemandFailure as Failure;

#[derive(Debug)]
pub(super) struct ReadmittedLineageEventPage {
    pub(super) rows: worth_execution::ExecutionArray<RelationalCheckpointLineageEventRow>,
}

pub(super) fn readmit(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    bytes: &[u8],
    descriptor: &RelationalCheckpointLineageEventPage,
) -> Result<ReadmittedLineageEventPage, Failure> {
    let unavailable = |cause| failure(descriptor, cause);
    let payload = member_frame::decode(bytes, descriptor.member, MemberKind::LineageEvents)
        .map_err(unavailable)?;
    let mut denial = None;
    let mut decoder = rmp_serde::Deserializer::new(std::io::Cursor::new(payload));
    let page = serde::Deserializer::deserialize_seq(
        &mut decoder,
        EventPageVisitor {
            execution,
            denial: &mut denial,
            descriptor,
        },
    )
    .map_err(|_| denial.unwrap_or_else(|| unavailable(Cause::Damaged)))?;
    if decoder.position() != payload.len() as u64
        || RelationalCheckpointLineageEventPage::from_rows(descriptor.member, page.rows.elements())
            .as_ref()
            != Some(descriptor)
        || page.rows.iter().any(|row| {
            row.publication_commit_id.0 == 0
                || row.event.commit().commit_id.0 == 0
                || row.event.commit().version_id.0 == 0
                || row.event.branch_id() != &row.event.commit().branch_id
                || !valid_event_shape(&row.event)
        })
    {
        return Err(unavailable(Cause::Damaged));
    }
    Ok(page)
}

struct EventPageVisitor<'request, 'scope, 'authority> {
    execution: &'request worth_execution::ExecutionRequest<'scope, 'authority>,
    denial: &'request mut Option<Failure>,
    descriptor: &'request RelationalCheckpointLineageEventPage,
}

impl<'de> Visitor<'de> for EventPageVisitor<'_, '_, '_> {
    type Value = ReadmittedLineageEventPage;

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("one ordered lineage event page of at most 256 rows")
    }

    fn visit_seq<A: SeqAccess<'de>>(mut self, mut input: A) -> Result<Self::Value, A::Error> {
        let width = input
            .size_hint()
            .filter(|width| *width == self.descriptor.event_ids.len() && (1..=256).contains(width))
            .ok_or_else(|| A::Error::custom("invalid lineage page width"))?;
        let mut rows = worth_execution::ExecutionArrayBuilder::allocate(
            width,
            self.execution.allocation_policy(),
        )
        .map_err(|denial| self.allocation_denial::<A::Error>(denial))?;
        while let Some(node) = input.next_element()? {
            if rows.len() == width {
                return Err(A::Error::custom("lineage page width drift"));
            }
            crate::contracts::execution::request_custody::check_request(self.execution).map_err(
                |cause| {
                    *self.denial = Some(Failure::Request(cause));
                    A::Error::custom("lineage page request stopped")
                },
            )?;
            rows.push(node)
                .map_err(|denial| self.allocation_denial::<A::Error>(denial))?;
        }
        if rows.len() != width {
            return Err(A::Error::custom("truncated lineage page"));
        }
        let rows = rows
            .seal()
            .map_err(|denial| self.allocation_denial::<A::Error>(denial))?;
        Ok(ReadmittedLineageEventPage { rows })
    }
}

impl EventPageVisitor<'_, '_, '_> {
    fn allocation_denial<E: Error>(
        &mut self,
        denial: worth_execution::ExecutionAllocationDenial,
    ) -> E {
        *self.denial = Some(match denial.classify() {
            worth_execution::ExecutionAllocationDenialClass::Request(cause) => {
                Failure::Request(cause.into())
            }
            worth_execution::ExecutionAllocationDenialClass::Physical(_) => {
                failure(self.descriptor, Cause::MemoryExhausted)
            }
        });
        E::custom("lineage page allocation refused")
    }
}

pub(super) fn failure(descriptor: &RelationalCheckpointLineageEventPage, cause: Cause) -> Failure {
    storage_failure(descriptor, cause).into()
}

pub(super) fn storage_failure(
    descriptor: &RelationalCheckpointLineageEventPage,
    cause: Cause,
) -> RelationalStorageUnavailable {
    RelationalStorageUnavailable {
        subject: RelationalStorageSubject::Lineage {
            first: crate::contracts::identity::data::primitives::LineageId(descriptor.first_node),
            last: crate::contracts::identity::data::primitives::LineageId(descriptor.last_node),
        },
        cause,
    }
}

fn valid_event_shape(event: &crate::contracts::lineage::data::events::LineageEventRecord) -> bool {
    use crate::contracts::lineage::data::events::LineageEventKind;
    let sources = event.sources();
    let targets = event.targets();
    let source_set = sources
        .iter()
        .copied()
        .collect::<std::collections::BTreeSet<_>>();
    let target_set = targets
        .iter()
        .copied()
        .collect::<std::collections::BTreeSet<_>>();
    let relation_binding_ok = match event.relation_id() {
        None => true,
        Some(_) => {
            event.subject() == crate::contracts::lineage::data::events::LineageSubject::Relation
                && matches!(
                    event.kind(),
                    LineageEventKind::Create | LineageEventKind::Restore
                )
                && targets.len() == 1
        }
    };
    let entity_binding_ok = match event.entity_id() {
        None => true,
        Some(_) => {
            event.subject() == crate::contracts::lineage::data::events::LineageSubject::Entity
                && matches!(
                    event.kind(),
                    LineageEventKind::Create | LineageEventKind::Restore
                )
                && targets.len() == 1
        }
    };
    source_set.len() == sources.len()
        && target_set.len() == targets.len()
        && source_set.is_disjoint(&target_set)
        && relation_binding_ok
        && entity_binding_ok
        && match event.kind() {
            LineageEventKind::Create => sources.is_empty() && targets.len() == 1,
            LineageEventKind::Replace => sources.len() == 1 && targets.len() == 1,
            LineageEventKind::Split => sources.len() == 1 && targets.len() >= 2,
            LineageEventKind::Merge => sources.len() >= 2 && targets.len() == 1,
            LineageEventKind::Retire => sources.len() == 1 && targets.is_empty(),
            LineageEventKind::Restore => sources.len() == 1 && targets.len() == 1,
        }
}

#[cfg(test)]
pub mod tests {
    use super::valid_event_shape;
    use crate::contracts::identity::data::{LineageId, VersionId};
    use crate::contracts::lineage::data::events::assemble_lineage_event_record;
    use crate::contracts::lineage::data::events::LineageEventRecordParts;
    use crate::contracts::lineage::data::{LineageEventKind, LineageEventRecord, LineageSubject};
    use worth_relational_branch::internal_port::history::data::{
        BranchId, CommitId, RelationalCommitReceipt,
    };

    fn restore_event(sources: Vec<LineageId>, targets: Vec<LineageId>) -> LineageEventRecord {
        let branch_id = BranchId("main".to_owned());
        assemble_lineage_event_record(LineageEventRecordParts {
            event_id: 1,
            commit: RelationalCommitReceipt {
                commit_id: CommitId(1),
                version_id: VersionId(1),
                branch_id: branch_id.clone(),
                parents: Vec::new(),
            },
            branch_id,
            kind: LineageEventKind::Restore,
            sources,
            targets,
            subject: LineageSubject::Entity,
            name_birth: None,
            kept_name: None,
            relation_id: None,
            entity_id: None,
        })
    }

    #[test]
    fn valid_event_shape_accepts_disjoint_one_to_one_restore_and_rejects_malformed() {
        assert!(valid_event_shape(&restore_event(
            vec![LineageId(1)],
            vec![LineageId(2)],
        )));
        assert!(!valid_event_shape(&restore_event(
            vec![],
            vec![LineageId(2)],
        )));
        assert!(!valid_event_shape(&restore_event(
            vec![LineageId(1)],
            vec![],
        )));
        assert!(!valid_event_shape(&restore_event(
            vec![LineageId(1), LineageId(2)],
            vec![LineageId(3)],
        )));
        assert!(!valid_event_shape(&restore_event(
            vec![LineageId(1)],
            vec![LineageId(2), LineageId(3)],
        )));
        assert!(!valid_event_shape(&restore_event(
            vec![LineageId(1)],
            vec![LineageId(1)],
        )));
    }
}
