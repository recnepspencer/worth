//! Exact member transport shares the selected region's terminal damage state.
use super::*;
use crate::durability::data::RelationalCheckpointMember;
use crate::durability::medium::RelationalDurableMemberCustody;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug)]
struct RegionMemberExport {
    owner: Arc<RelationalRegionResidency>,
    sources: BTreeMap<RelationalDurableMemberLocator, Arc<dyn RelationalDurableColdSource>>,
}
impl RelationalRegionResidency {
    pub(crate) fn checkpoint_export_sources(
        self: &Arc<Self>,
        members: &[RelationalCheckpointMember],
        fallback: Option<Arc<dyn RelationalDurableColdSource>>,
    ) -> Result<
        BTreeMap<
            RelationalDurableMemberLocator,
            (
                RelationalStorageSubject,
                Arc<dyn RelationalDurableColdSource>,
            ),
        >,
        RelationalStorageUnavailable,
    > {
        self.check_damage()?;
        let required = members
            .iter()
            .filter_map(|member| match member {
                RelationalCheckpointMember::Carried { locator } => Some(*locator),
                RelationalCheckpointMember::New { .. } => None,
            })
            .collect::<BTreeSet<_>>();
        let mut sources = BTreeMap::new();
        if let Some(source) = fallback {
            sources.extend(required.iter().map(|member| (*member, source.clone())));
        }
        if let Some(cold) = &self.backing {
            let mut backed = cold.history_members.members(cold.history);
            backed.insert(cold.head);
            sources.extend(
                backed
                    .intersection(&required)
                    .map(|member| (*member, cold.source.clone())),
            );
        }
        if let Some(lease) = self.lease_checkpoint_contents()? {
            let partition = lease.checkpoint_contents();
            for prefix in partition
                .entity_arena
                .history_prefixes
                .iter()
                .flatten()
                .chain(partition.relation_arena.history_prefixes.iter().flatten())
            {
                sources.extend(
                    prefix
                        .history_members
                        .members(prefix.member)
                        .intersection(&required)
                        .map(|member| (*member, prefix.source.clone())),
                );
            }
        }
        if sources.len() != required.len() {
            return Err(self.failure(Cause::SourceUnavailable));
        }
        let export: Arc<dyn RelationalDurableColdSource> = Arc::new(RegionMemberExport {
            owner: self.clone(),
            sources,
        });
        Ok(required
            .into_iter()
            .map(|member| {
                (
                    member,
                    (
                        RelationalStorageSubject::Region(self.partition_id),
                        export.clone(),
                    ),
                )
            })
            .collect())
    }
}
impl RelationalDurableColdSource for RegionMemberExport {
    fn retain_members(
        &self,
        members: &[RelationalDurableMemberLocator],
    ) -> Result<RelationalDurableMemberCustody, Cause> {
        let custody = members
            .iter()
            .map(|member| {
                self.sources
                    .get(member)
                    .ok_or(Cause::SourceUnavailable)?
                    .retain_members(&[*member])
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(RelationalDurableMemberCustody::new(custody))
    }
    fn read_member(
        &self,
        member: RelationalDurableMemberLocator,
        limit: u64,
    ) -> Result<Vec<u8>, Cause> {
        let _state = self
            .owner
            .state
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        self.owner.check_damage().map_err(|failure| failure.cause)?;
        let source = self.sources.get(&member).ok_or(Cause::SourceUnavailable)?;
        let result = if member.length > limit {
            Err(Cause::LimitExceeded {
                length: member.length,
                limit,
            })
        } else {
            source.read_member(member, limit)
        }
        .and_then(|bytes| {
            if crate::durability::log::native_checkpoint_codec::member_frame::locator(&bytes)
                != member
            {
                return Err(Cause::IntegrityMismatch);
            }
            Ok(bytes)
        });
        if let Err(cause) = &result
            && cause.is_proven_damage()
        {
            let _ = self.owner.damage.set(self.owner.failure(*cause));
        }
        result
    }
    fn read_commit(
        &self,
        _: crate::publication::patch::data::PatchStreamPosition,
        _: u64,
    ) -> Result<Vec<u8>, Cause> {
        Err(Cause::SourceUnavailable)
    }
}
