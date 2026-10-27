//! Index transport and generation admission share one image failure state.
use super::*;
use crate::durability::medium::{
    RelationalDurableColdSource, RelationalDurableMemberCustody, RelationalDurableMemberLocator,
};
impl IndexingSubsystem {
    pub(crate) fn checkpoint_export_sources(
        &self,
    ) -> std::collections::BTreeMap<
        RelationalDurableMemberLocator,
        (
            RelationalStorageSubject,
            Arc<dyn RelationalDurableColdSource>,
        ),
    > {
        let Some(image) = self.state.read().cold_checkpoint.clone() else {
            return Default::default();
        };
        let member = image.directory.manifest.derived_indexes;
        let subject = RelationalStorageSubject::Index(
            image
                .directory
                .manifest
                .index_definitions
                .first()
                .map_or(crate::indexes::data::DerivedIndexId(0), |definition| {
                    definition.index_id
                }),
        );
        let source = image as Arc<dyn RelationalDurableColdSource>;
        [(member, (subject, source))].into_iter().collect()
    }
}
impl RelationalDurableColdSource for DeferredIndexImage {
    fn retain_members(
        &self,
        members: &[RelationalDurableMemberLocator],
    ) -> Result<RelationalDurableMemberCustody, Cause> {
        self.directory.source.retain_members(members)
    }
    fn read_member(
        &self,
        member: RelationalDurableMemberLocator,
        limit: u64,
    ) -> Result<Vec<u8>, Cause> {
        if member != self.directory.manifest.derived_indexes {
            return Err(Cause::SourceUnavailable);
        }
        let mut admitted = self
            .admitted
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        if let Some(Err(failure)) = &*admitted {
            return Err(failure.cause);
        }
        let result = if member.length > limit {
            Err(Cause::LimitExceeded {
                length: member.length,
                limit,
            })
        } else {
            self.directory.source.read_member(member, limit)
        }
        .and_then(|bytes| {
            member_frame::decode(&bytes, member, MemberKind::DerivedIndex)?;
            Ok(bytes)
        });
        if let Err(cause) = &result
            && cause.is_proven_damage()
        {
            *admitted = Some(Err(RelationalStorageUnavailable {
                subject: RelationalStorageSubject::Index(
                    self.directory
                        .manifest
                        .index_definitions
                        .first()
                        .map_or(crate::indexes::data::DerivedIndexId(0), |definition| {
                            definition.index_id
                        }),
                ),
                cause: *cause,
            }));
        }
        result
    }
    fn read_commit(
        &self,
        position: crate::publication::patch::data::PatchStreamPosition,
        limit: u64,
    ) -> Result<Vec<u8>, Cause> {
        self.directory.source.read_commit(position, limit)
    }
}
