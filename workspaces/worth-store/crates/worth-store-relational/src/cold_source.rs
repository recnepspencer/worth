//! Read-only placement adapter. Relational verifies its own member and artifact
//! frames after Store has verified physical integrity and current membership.
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc, RwLock, Weak,
};

use worth_relational::facade::durability::{
    RelationalDurableColdSource, RelationalDurableMemberCustody, RelationalDurableMemberLocator,
};
use worth_relational::facade::publication::PatchStreamPosition;
use worth_relational::facade::storage::RelationalStorageUnavailableCause as Cause;
use worth_store::physical_runtime::{
    AnchoredRecordMemberDamage, AnchoredRecordMemberLocator, AnchoredRecordMemberReadFailure,
    AnchoredRecordName, OrderedJournalReadFailure, RecordReadDenial, RecordReadError,
    ServingPhysicalRuntime,
};

mod member_custody;
#[cfg(test)]
mod member_custody_tests;
#[cfg(test)]
mod test_store;
#[cfg(test)]
mod tests;
pub(crate) use member_custody::RetainedMemberPublication;

/// Calls through the home's demand-read port, including typed refusals.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RelationalStoreColdReadCounters {
    pub member_reads: u64,
    pub commit_reads: u64,
}

pub(crate) struct StoreColdSource {
    weak_self: Weak<Self>,
    serving: Arc<ServingPhysicalRuntime>,
    members: RwLock<(
        u64,
        std::collections::BTreeMap<RelationalDurableMemberLocator, AnchoredRecordMemberLocator>,
    )>,
    custody: member_custody::MemberCustodyRegistry,
    member_reads: AtomicU64,
    commit_reads: AtomicU64,
    #[cfg(feature = "certification-test-authority")]
    commit_read_observer:
        Option<Box<dyn Fn(PatchStreamPosition, RelationalStoreColdReadCounters) + Send + Sync>>,
}

impl std::fmt::Debug for StoreColdSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StoreColdSource").finish_non_exhaustive()
    }
}

impl StoreColdSource {
    pub(crate) fn new(serving: Arc<ServingPhysicalRuntime>) -> Arc<Self> {
        Arc::new_cyclic(|weak_self| Self {
            weak_self: weak_self.clone(),
            serving,
            members: RwLock::new((0, Default::default())),
            custody: Default::default(),
            member_reads: AtomicU64::new(0),
            commit_reads: AtomicU64::new(0),
            #[cfg(feature = "certification-test-authority")]
            commit_read_observer: None,
        })
    }
    /// A verified selected membership supplied by open or completed publication.
    pub(crate) fn select_members(&self, generation: u64, members: &[AnchoredRecordMemberLocator]) {
        let selected = members
            .iter()
            .map(|member| {
                (
                    RelationalDurableMemberLocator {
                        sha256: member.sha256(),
                        length: member.length(),
                    },
                    *member,
                )
            })
            .collect();
        self.custody.select(|| {
            let mut current = self
                .members
                .write()
                .unwrap_or_else(|error| error.into_inner());
            if generation >= current.0 {
                *current = (generation, selected);
            }
        });
    }
    pub(crate) fn counters(&self) -> RelationalStoreColdReadCounters {
        RelationalStoreColdReadCounters {
            member_reads: self.member_reads.load(Ordering::Relaxed),
            commit_reads: self.commit_reads.load(Ordering::Relaxed),
        }
    }
    #[cfg(feature = "certification-test-authority")]
    pub(crate) fn new_with_commit_read_observer(
        serving: Arc<ServingPhysicalRuntime>,
        observer: impl Fn(PatchStreamPosition, RelationalStoreColdReadCounters) + Send + Sync + 'static,
    ) -> Arc<Self> {
        let observer = Box::new(observer);
        Arc::new_cyclic(|weak_self| Self {
            weak_self: weak_self.clone(),
            serving,
            members: RwLock::new((0, Default::default())),
            custody: Default::default(),
            member_reads: AtomicU64::new(0),
            commit_reads: AtomicU64::new(0),
            commit_read_observer: Some(observer),
        })
    }
    pub(crate) fn protect_member_selection(
        &self,
        next: impl IntoIterator<Item = RelationalDurableMemberLocator>,
    ) -> Result<RetainedMemberPublication, Cause> {
        self.custody.protect_selection(next)
    }
    pub(crate) fn member_locators(&self) -> Vec<RelationalDurableMemberLocator> {
        self.members
            .read()
            .unwrap_or_else(|error| error.into_inner())
            .1
            .keys()
            .copied()
            .collect()
    }

    fn retain_custody_owner(
        &self,
        members: &[RelationalDurableMemberLocator],
    ) -> Result<member_custody::MemberCustodyOwner, Cause> {
        self.custody
            .retain(members, self.serving.clone(), |selected| {
                let current = self
                    .members
                    .read()
                    .unwrap_or_else(|error| error.into_inner());
                if selected.iter().all(|member| current.1.contains_key(member)) {
                    Ok(())
                } else {
                    Err(Cause::MissingMember)
                }
            })
    }
}

impl RelationalDurableColdSource for StoreColdSource {
    fn validate_member_custody_retirement(
        &self,
        original: &(dyn std::any::Any + Send + Sync),
    ) -> Result<(), worth_relational::facade::durability::RelationalDurableMemberRetirementRefusal>
    {
        self.validate_custody_owner(original)
    }

    fn advance_member_custody_retirement(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        original: &mut (dyn std::any::Any + Send + Sync),
    ) -> worth_relational::facade::durability::RelationalDurableMemberRetirementObservation {
        self.advance_custody_owner(execution, original)
    }

    fn retain_members(
        &self,
        members: &[RelationalDurableMemberLocator],
    ) -> Result<RelationalDurableMemberCustody, Cause> {
        Ok(RelationalDurableMemberCustody::new_from_source(
            self.retain_custody_owner(members)?,
            self.weak_self.clone(),
        ))
    }

    fn read_member(
        &self,
        locator: RelationalDurableMemberLocator,
        limit: u64,
    ) -> Result<Vec<u8>, Cause> {
        self.member_reads.fetch_add(1, Ordering::Relaxed);
        if locator.length > limit {
            return Err(Cause::LimitExceeded {
                length: locator.length,
                limit,
            });
        }
        let records = self
            .serving
            .anchored_records()
            .map_err(|_| Cause::SourceUnavailable)?;
        let anchor = AnchoredRecordName::new("home.checkpoint").expect("static anchor");
        let placed = self
            .members
            .read()
            .unwrap_or_else(|error| error.into_inner())
            .1
            .get(&locator)
            .copied()
            .ok_or(Cause::MissingMember)?;
        records
            .read_member(anchor, placed, limit)
            .map_err(|failure| match failure {
                AnchoredRecordMemberReadFailure::NotCurrentMember => Cause::MissingMember,
                AnchoredRecordMemberReadFailure::LimitExceeded { length } => {
                    Cause::LimitExceeded { length, limit }
                }
                AnchoredRecordMemberReadFailure::Damaged(
                    AnchoredRecordMemberDamage::DigestMismatch,
                ) => Cause::IntegrityMismatch,
                AnchoredRecordMemberReadFailure::Damaged(_) => Cause::Damaged,
                // Failure to verify the selecting anchor proves nothing about
                // this member's bytes. It must not poison every region cell.
                AnchoredRecordMemberReadFailure::Value(_) => Cause::SourceUnavailable,
                AnchoredRecordMemberReadFailure::Open(error) => record_failure(error),
                _ => Cause::SourceUnavailable,
            })
    }

    fn read_commit(&self, position: PatchStreamPosition, limit: u64) -> Result<Vec<u8>, Cause> {
        self.commit_reads.fetch_add(1, Ordering::Relaxed);
        let bytes = self
            .serving
            .canonical_commits()
            .map_err(|_| Cause::SourceUnavailable)?
            .read(position.0, limit)
            .map(|record| record.into_bytes())
            .map_err(|failure| match failure {
                OrderedJournalReadFailure::NotFound { .. } => Cause::MissingMember,
                OrderedJournalReadFailure::Open(error) => record_failure(error),
                OrderedJournalReadFailure::Damaged => Cause::Damaged,
                OrderedJournalReadFailure::LimitExceeded { payload_bytes } => {
                    Cause::LimitExceeded {
                        length: payload_bytes,
                        limit,
                    }
                }
                _ => Cause::SourceUnavailable,
            })?;
        #[cfg(feature = "certification-test-authority")]
        if let Some(observer) = &self.commit_read_observer {
            observer(position, self.counters());
        }
        Ok(bytes)
    }
}

fn record_failure(error: RecordReadError) -> Cause {
    match error.denial() {
        RecordReadDenial::RecordNotFound => Cause::MissingMember,
        RecordReadDenial::ArtifactDamaged | RecordReadDenial::FormatMismatch => Cause::Damaged,
        _ => Cause::SourceUnavailable,
    }
}
