//! Dictionary export uses the same per-member damage ledger as symbol admission.
use super::*;
use crate::durability::log::native_checkpoint_codec::member_frame::{self, MemberKind};
use crate::durability::medium::RelationalDurableMemberCustody;

impl RuntimeSymbolResidency {
    fn member_subjects(
        &self,
    ) -> BTreeMap<RelationalDurableMemberLocator, RelationalStorageSubject> {
        let mut subjects = self
            .by_value
            .iter()
            .flat_map(|bucket| bucket.pages.iter())
            .map(|member| (*member, SymbolPageSelection::HashBucket(0).subject()))
            .collect::<BTreeMap<_, _>>();
        subjects.extend(self.by_id.iter().map(|page| {
            (
                page.member,
                RelationalStorageSubject::Symbols {
                    first: Symbol(page.first as u32),
                    last: Symbol(page.last as u32),
                },
            )
        }));
        subjects
    }
}
#[derive(Debug)]
struct SymbolMemberExport {
    owner: Arc<RuntimeSymbolResidency>,
    member: RelationalDurableMemberLocator,
    subject: RelationalStorageSubject,
}
impl RuntimeSymbolResidency {
    pub(in crate::runtime::state::subsystems::services) fn export_sources(
        self: &Arc<Self>,
    ) -> BTreeMap<
        RelationalDurableMemberLocator,
        (
            RelationalStorageSubject,
            Arc<dyn RelationalDurableColdSource>,
        ),
    > {
        self.member_subjects()
            .into_iter()
            .map(|(member, subject)| {
                let source: Arc<dyn RelationalDurableColdSource> = Arc::new(SymbolMemberExport {
                    owner: self.clone(),
                    member,
                    subject,
                });
                (member, (subject, source))
            })
            .collect()
    }
}
impl RelationalDurableColdSource for SymbolMemberExport {
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
        if member != self.member {
            return Err(Cause::SourceUnavailable);
        }
        let subject = self.subject;
        let mut pages = self
            .owner
            .pages
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        if let Some(Err(failure)) = pages.get(&member) {
            return Err(failure.cause);
        }
        let result = if member.length > limit {
            Err(Cause::LimitExceeded {
                length: member.length,
                limit,
            })
        } else {
            self.owner.source.read_member(member, limit)
        }
        .and_then(|bytes| {
            member_frame::decode(&bytes, member, MemberKind::Symbols)?;
            Ok(bytes)
        });
        if let Err(cause) = &result
            && cause.is_proven_damage()
        {
            pages.insert(member, Err(unavailable(subject, *cause)));
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
