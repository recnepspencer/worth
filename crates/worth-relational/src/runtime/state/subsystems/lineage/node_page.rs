//! Request-funded readmission of one authenticated, bounded node-range page.
use crate::durability::data::RelationalCheckpointPage;
use crate::durability::log::native_checkpoint_codec::member_frame::{self, MemberKind};
use crate::lineage::data::LineageNode;
use crate::storage::data::{
    RelationalStorageSubject, RelationalStorageUnavailable,
    RelationalStorageUnavailableCause as Cause,
};
use crate::storage::residency::RelationalResidencyDemandFailure as Failure;
use serde::de::{Error, SeqAccess, Visitor};

#[derive(Debug)]
pub(super) struct ReadmittedLineageNodePage {
    pub(super) nodes: worth_execution::ExecutionArray<LineageNode>,
}

pub(super) fn readmit(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    bytes: &[u8],
    descriptor: &RelationalCheckpointPage,
) -> Result<ReadmittedLineageNodePage, Failure> {
    let unavailable = |cause| failure(descriptor, cause);
    let payload =
        member_frame::decode(bytes, descriptor.member, MemberKind::Lineage).map_err(unavailable)?;
    let mut denial = None;
    let mut decoder = rmp_serde::Deserializer::new(std::io::Cursor::new(payload));
    let page = serde::Deserializer::deserialize_seq(
        &mut decoder,
        NodePageVisitor {
            execution,
            denial: &mut denial,
            descriptor,
        },
    )
    .map_err(|_| denial.unwrap_or_else(|| unavailable(Cause::Damaged)))?;
    if decoder.position() != payload.len() as u64
        || page
            .nodes
            .first()
            .is_none_or(|node| node.lineage_id().0 != descriptor.first)
        || page
            .nodes
            .last()
            .is_none_or(|node| node.lineage_id().0 != descriptor.last)
        || page
            .nodes
            .iter()
            .any(|node| node.lineage_id().0 == 0 || node.lineage_id().0 >= u64::MAX - 1)
        || page
            .nodes
            .windows(2)
            .any(|pair| pair[0].lineage_id() >= pair[1].lineage_id())
    {
        return Err(unavailable(Cause::Damaged));
    }
    if RelationalCheckpointPage::from_ids(
        page.nodes.iter().map(|node| node.lineage_id().0),
        descriptor.member,
    )
    .as_ref()
        != Some(descriptor)
    {
        return Err(unavailable(Cause::Damaged));
    }
    let mut entities = [None; 256];
    for (slot, node) in page.nodes.iter().enumerate() {
        entities[slot] = Some(node.entity_id());
    }
    let entities = &mut entities[..page.nodes.len()];
    entities.sort_unstable();
    if entities.windows(2).any(|pair| pair[0] == pair[1]) {
        return Err(unavailable(Cause::Damaged));
    }
    Ok(page)
}

struct NodePageVisitor<'request, 'scope, 'authority> {
    execution: &'request worth_execution::ExecutionRequest<'scope, 'authority>,
    denial: &'request mut Option<Failure>,
    descriptor: &'request RelationalCheckpointPage,
}

impl<'de> Visitor<'de> for NodePageVisitor<'_, '_, '_> {
    type Value = ReadmittedLineageNodePage;

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("one ordered lineage page of at most 256 nodes")
    }

    fn visit_seq<A: SeqAccess<'de>>(mut self, mut input: A) -> Result<Self::Value, A::Error> {
        let width = input
            .size_hint()
            .filter(|width| (1..=256).contains(width))
            .ok_or_else(|| A::Error::custom("invalid lineage page width"))?;
        let mut nodes = worth_execution::ExecutionArrayBuilder::allocate(
            width,
            self.execution.allocation_policy(),
        )
        .map_err(|denial| self.allocation_denial::<A::Error>(denial))?;
        while let Some(node) = input.next_element()? {
            if nodes.len() == width {
                return Err(A::Error::custom("lineage page width drift"));
            }
            crate::execution::check_request(self.execution).map_err(|cause| {
                *self.denial = Some(Failure::Request(cause));
                A::Error::custom("lineage page request stopped")
            })?;
            nodes
                .push(node)
                .map_err(|denial| self.allocation_denial::<A::Error>(denial))?;
        }
        if nodes.len() != width {
            return Err(A::Error::custom("truncated lineage page"));
        }
        let nodes = nodes
            .seal()
            .map_err(|denial| self.allocation_denial::<A::Error>(denial))?;
        Ok(ReadmittedLineageNodePage { nodes })
    }
}

impl NodePageVisitor<'_, '_, '_> {
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

pub(super) fn failure(descriptor: &RelationalCheckpointPage, cause: Cause) -> Failure {
    RelationalStorageUnavailable {
        subject: RelationalStorageSubject::Lineage {
            first: crate::identity::data::LineageId(descriptor.first),
            last: crate::identity::data::LineageId(descriptor.last),
        },
        cause,
    }
    .into()
}
