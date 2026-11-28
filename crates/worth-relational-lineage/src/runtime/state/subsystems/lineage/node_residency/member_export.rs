//! Node-frame transport and point demand share terminal page damage.
use super::*;
use worth_relational_contracts::internal_port::storage::data::storage_unavailable::RelationalStorageSubject;
use worth_relational_storage::internal_port::durability::medium::cold_source::RelationalDurableMemberLocator;
use worth_relational_storage::internal_port::durability::medium::member_custody::RelationalDurableMemberCustody;
#[derive(Debug)]
struct NodeMemberExport {
    owner: Arc<RuntimeLineageNodeResidency>,
    page: Arc<NodePageCell>,
}
impl RuntimeLineageNodeResidency {
    pub(in crate::internal_port::runtime::state::subsystems::lineage) fn export_sources(
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
        self.owner.custody.source()?.retain_members(members)
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
        page.export_member(|| self.owner.custody.source(), limit)
    }
    fn read_commit(
        &self,
        position: worth_relational_contracts::internal_port::publication::patch::data::patch_position::PatchStreamPosition,
        limit: u64,
    ) -> Result<Vec<u8>, Cause> {
        self.owner.custody.source()?.read_commit(position, limit)
    }
}
