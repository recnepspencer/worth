//! Request-funded readmission of one event payload and its complete range routing.
use crate::durability::data::{
    RelationalCheckpointLineageEventPage, RelationalCheckpointLineageEventRow,
};
use crate::durability::log::native_checkpoint_codec::member_frame::{self, MemberKind};
use crate::storage::data::{
    RelationalStorageSubject, RelationalStorageUnavailable,
    RelationalStorageUnavailableCause as Cause,
};
use crate::storage::residency::RelationalResidencyDemandFailure as Failure;
use serde::de::{Error, SeqAccess, Visitor};

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
            crate::execution::check_request(self.execution).map_err(|cause| {
                *self.denial = Some(Failure::Request(cause));
                A::Error::custom("lineage page request stopped")
            })?;
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
    RelationalStorageUnavailable {
        subject: RelationalStorageSubject::Lineage {
            first: crate::identity::data::LineageId(descriptor.first_node),
            last: crate::identity::data::LineageId(descriptor.last_node),
        },
        cause,
    }
    .into()
}

fn valid_event_shape(event: &crate::lineage::data::LineageEventRecord) -> bool {
    use crate::lineage::data::LineageEventKind;
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
    source_set.len() == sources.len()
        && target_set.len() == targets.len()
        && source_set.is_disjoint(&target_set)
        && match event.kind() {
            LineageEventKind::Create => sources.is_empty() && targets.len() == 1,
            LineageEventKind::Replace => sources.len() == 1 && targets.len() == 1,
            LineageEventKind::Split => sources.len() == 1 && targets.len() >= 2,
            LineageEventKind::Merge => sources.len() >= 2 && targets.len() == 1,
            LineageEventKind::Retire => sources.len() == 1 && targets.is_empty(),
        }
}
