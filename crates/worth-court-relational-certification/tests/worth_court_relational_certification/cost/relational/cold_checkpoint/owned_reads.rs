//! Original immutable member snapshots, funded by the actual requesting owner.
use super::*;
use worth_execution::ExecutionImmutableBytes;
pub(super) use worth_testkit_relational::durable_medium::OwnedNativeReadSnapshot as Snapshot;
use worth_testkit_relational::durable_medium::OwnedSnapshotDenial;
use RelationalDurableOwnedReadRefusal as Refusal;

pub(super) fn snapshot(original: &(dyn std::any::Any + Send + Sync)) -> Result<&Snapshot, Refusal> {
    original
        .downcast_ref::<Snapshot>()
        .ok_or(Refusal::ForeignProtection)
}

impl ColdCheckpointSource {
    pub(super) fn snapshot(
        &self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Box<Snapshot>, Refusal> {
        Snapshot::capture(
            request,
            self.commits
                .iter()
                .map(|(position, bytes)| (position.0, bytes.as_slice())),
            std::iter::empty(),
            &self.original_members,
            None,
        )
        .map_err(failure)
    }
    pub(super) fn read_owned_member(
        &self,
        request: &ExecutionRequest<'_, '_>,
        snapshot: &Snapshot,
        locator: RelationalDurableMemberLocator,
        limit: u64,
    ) -> Result<ExecutionImmutableBytes, Refusal> {
        request
            .check_live()
            .map_err(|cause| Refusal::Request(cause.into()))?;
        if locator.length > limit {
            return Err(Refusal::Storage(Cause::LimitExceeded {
                length: locator.length,
                limit,
            }));
        }
        self.reads.lock().unwrap().push(locator);
        if self.damaged.load(Ordering::SeqCst) {
            return Err(Refusal::Storage(Cause::Damaged));
        }
        if self.unavailable.load(Ordering::SeqCst) {
            return Err(Refusal::Storage(Cause::SourceUnavailable));
        }
        let bytes = snapshot
            .member(request, locator.sha256, locator.length, limit)
            .map_err(failure)?;
        self.bytes.fetch_add(bytes.len() as u64, Ordering::SeqCst);
        Ok(bytes)
    }
}

pub(super) fn failure(cause: OwnedSnapshotDenial) -> Refusal {
    match cause {
        OwnedSnapshotDenial::Request(cause) => Refusal::Request(cause.into()),
        OwnedSnapshotDenial::Buffer(cause) => Refusal::Buffer(cause),
        OwnedSnapshotDenial::Missing => Refusal::Storage(Cause::MissingMember),
        OwnedSnapshotDenial::Integrity => Refusal::Storage(Cause::IntegrityMismatch),
        OwnedSnapshotDenial::Limit { length, limit } => {
            Refusal::Storage(Cause::LimitExceeded { length, limit })
        }
        OwnedSnapshotDenial::Capacity => Refusal::Storage(Cause::MemoryExhausted),
    }
}
