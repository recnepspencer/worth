//! Node-frame transport and point demand share terminal page damage.
use super::*;
use crate::durability::log::native_checkpoint_codec::member_frame::{self, MemberKind};
use crate::durability::medium::{RelationalDurableMemberCustody, RelationalDurableMemberLocator};
use crate::storage::data::RelationalStorageSubject;
#[derive(Debug)]
struct NodeMemberExport {
    owner: Arc<RuntimeLineageNodeResidency>,
    page: Arc<NodePageCell>,
}
impl RuntimeLineageNodeResidency {
    pub(in crate::runtime::state::subsystems::lineage) fn export_sources(
        self: &Arc<Self>,
    ) -> std::collections::BTreeMap<
        RelationalDurableMemberLocator,
        (
            RelationalStorageSubject,
            Arc<dyn RelationalDurableColdSource>,
        ),
    > {
        self.pages
            .iter()
            .map(|page| {
                let subject = RelationalStorageSubject::Lineage {
                    first: LineageId(page.descriptor.first),
                    last: LineageId(page.descriptor.last),
                };
                let source: Arc<dyn RelationalDurableColdSource> = Arc::new(NodeMemberExport {
                    owner: self.clone(),
                    page: page.clone(),
                });
                (page.descriptor.member, (subject, source))
            })
            .collect()
    }
}
impl RelationalDurableColdSource for NodeMemberExport {
    fn retain_members(
        &self,
        members: &[RelationalDurableMemberLocator],
    ) -> Result<RelationalDurableMemberCustody, Cause> {
        self.owner.source.retain_members(members)
    }
    fn read_member(
        &self,
        member: RelationalDurableMemberLocator,
        limit: u64,
    ) -> Result<Vec<u8>, Cause> {
        let page = &self.page;
        if member != page.descriptor.member {
            return Err(Cause::SourceUnavailable);
        }
        let mut state = page.state.lock().unwrap_or_else(|error| error.into_inner());
        if let NodePageState::Damaged(failure) = &*state {
            return Err(failure.cause);
        }
        let limit = limit.min(1 << 20);
        let result = if member.length > limit {
            Err(Cause::LimitExceeded {
                length: member.length,
                limit,
            })
        } else {
            self.owner.source.read_member(member, limit)
        }
        .and_then(|bytes| {
            member_frame::decode(&bytes, member, MemberKind::Lineage)?;
            Ok(bytes)
        });
        if let Err(cause) = &result
            && cause.is_proven_damage()
        {
            let Failure::StorageUnavailable(failure) = node_page::failure(&page.descriptor, *cause)
            else {
                unreachable!()
            };
            *state = NodePageState::Damaged(failure);
        }
        result
    }
    fn read_commit(
        &self,
        position: crate::publication::patch::data::PatchStreamPosition,
        limit: u64,
    ) -> Result<Vec<u8>, Cause> {
        self.owner.source.read_commit(position, limit)
    }
}
