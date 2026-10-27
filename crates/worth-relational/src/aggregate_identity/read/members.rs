//! Typed projections of one retained membership body, never carrier coercions.
use super::*;
impl AggregateRead<EntityId> {
    pub fn anchor(&self) -> EntityId {
        self.member_set()
            .first()
            .and_then(NativeMemberKey::entity)
            .expect("physical read door validates entity-domain source")
    }
    pub fn members(&self) -> impl ExactSizeIterator<Item = EntityId> + '_ {
        self.member_set().iter().map(|member| {
            member
                .entity()
                .expect("physical read door validates entity-domain source")
        })
    }
    pub fn contains(&self, member: EntityId) -> bool {
        self.member_set().contains(&NativeMemberKey::Entity(member))
    }
}
impl AggregateRead<NativeMemberKey> {
    pub fn anchor(&self) -> &NativeMemberKey {
        self.member_set()
            .first()
            .expect("complete nonempty projected cohort")
    }
    pub fn members(&self) -> impl ExactSizeIterator<Item = &NativeMemberKey> {
        self.member_set().iter()
    }
    pub fn contains(&self, member: &NativeMemberKey) -> bool {
        self.member_set().contains(member)
    }
}
