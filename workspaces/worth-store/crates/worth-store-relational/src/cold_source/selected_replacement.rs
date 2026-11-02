//! One original checkpoint's fixed selection and resumable placement cutover.
use super::selection::RelationalStoreMemberSelectionDenial as Denial;
use super::{
    selection::{MemberMap, Placement, SelectedMembers},
    StoreColdSource,
};
use std::sync::{TryLockError, Weak};
use worth_execution::ExecutionRequest;
use worth_relational::facade::durability::RelationalDurableMemberLocator;
use worth_store::physical_runtime::{
    AnchoredRecordReadFailure as ReadFailure, AnchoredRecordReadMembership,
    AnchoredRecordReadRetirement, AnchoredRecordReadRetirementProgress, AnchoredRecordVersion,
    PhysicalCheckpointReadProtectionDenial as Capture, RetiredAnchoredRecordReadMembership,
};

#[derive(Default)]
pub(crate) struct SelectedReplacement {
    issuer: Option<Weak<StoreColdSource>>,
    expected: Option<AnchoredRecordVersion>,
    previous: Option<Option<AnchoredRecordVersion>>,
    read: Option<AnchoredRecordReadMembership>,
    entries: MemberMap,
    next_member: usize,
    displaced: Option<SelectedMembers>,
    retirement: Option<AnchoredRecordReadRetirement>,
    _retired: Option<RetiredAnchoredRecordReadMembership>,
    installed: bool,
    complete: bool,
}
impl SelectedReplacement {
    pub(crate) fn advance(
        &mut self,
        source: &StoreColdSource,
        expected: AnchoredRecordVersion,
        required: &[RelationalDurableMemberLocator],
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), Denial> {
        if self
            .issuer
            .as_ref()
            .is_some_and(|issuer| issuer.as_ptr() != source as *const StoreColdSource)
            || self.expected.is_some_and(|original| original != expected)
        {
            return Err(Denial::DifferentSelection);
        }
        self.issuer.get_or_insert_with(|| source.weak_self.clone());
        self.expected = Some(expected);
        if self.complete {
            return Ok(());
        }
        request.consult().map_err(Denial::Execution)?;
        if self.previous.is_none() {
            let current = try_read(&source.members)?;
            if current.version == Some(expected)
                || current
                    .version
                    .is_some_and(|old| old.generation() > expected.generation())
            {
                return Err(Denial::DifferentSelection);
            }
            self.previous = Some(current.version);
        }
        if !self.installed {
            self.prepare(source, expected, required, request)?;
            if self.complete {
                return Ok(());
            }
            self.install(source, request)?;
        }
        self.retire_previous(source, request)
    }
    fn prepare(
        &mut self,
        source: &StoreColdSource,
        expected: AnchoredRecordVersion,
        required: &[RelationalDurableMemberLocator],
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), Denial> {
        if self.displaced.is_some() {
            return Ok(());
        }
        if self.read.is_none() {
            self.read = Some(read_expected(source, expected, request)?);
        }
        let read = self
            .read
            .as_ref()
            .expect("retained verified replacement read");
        let records = source
            .serving()
            .anchored_records()
            .map_err(Denial::Unavailable)?;
        let membership = records
            .validate_read_membership(read)
            .map_err(Denial::Read)?;
        while let Some(member) = membership.members().get(self.next_member) {
            let key = RelationalDurableMemberLocator {
                sha256: member.sha256(),
                length: member.length(),
            };
            self.entries
                .insert_cloned_native_for_request(
                    &key,
                    &Placement {
                        index: self.next_member,
                        locator: *member,
                    },
                    *request,
                )
                .map_err(Denial::Allocation)?;
            self.next_member += 1;
        }
        for required in required {
            request.consult().map_err(Denial::Execution)?;
            if !self.entries.contains_key(required) {
                return Err(Denial::DifferentSelection);
            }
        }
        if self.previous == Some(None) {
            // First admission uses the original guard, never replacement of
            // an ordinary same-version description-only map.
            source.install_initial_built(read, &mut self.entries, request)?;
            self.installed = true;
            self.complete = true;
            return Ok(());
        }
        self.displaced = Some(SelectedMembers {
            version: Some(expected),
            entries: std::mem::take(&mut self.entries),
            read: self.read.take(),
        });
        Ok(())
    }
    fn install(
        &mut self,
        source: &StoreColdSource,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), Denial> {
        let previous = self.previous.expect("captured original placement");
        let candidate = self
            .displaced
            .as_mut()
            .expect("prepared original replacement");
        let installed = &mut self.installed;
        source
            .custody
            .try_select(|| {
                let mut current = try_write(&source.members)?;
                if current.version != previous {
                    return Err(Denial::DifferentSelection);
                }
                request.consult().map_err(Denial::Execution)?;
                std::mem::swap(&mut *current, candidate);
                *installed = true;
                Ok(())
            })
            .map_err(|_| Denial::Busy)??;
        Ok(())
    }
    fn retire_previous(
        &mut self,
        source: &StoreColdSource,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), Denial> {
        if self.retirement.is_none() {
            self.retirement = self
                .displaced
                .as_mut()
                .and_then(|old| old.read.take())
                .map(AnchoredRecordReadMembership::into_retirement);
        }
        if let Some(task) = self.retirement.as_mut() {
            let records = source
                .serving()
                .anchored_records()
                .map_err(Denial::Unavailable)?;
            loop {
                match task
                    .advance(&records, request)
                    .map_err(Denial::Retirement)?
                {
                    AnchoredRecordReadRetirementProgress::Advanced => {}
                    AnchoredRecordReadRetirementProgress::Complete => break,
                }
            }
            self._retired = Some(
                self.retirement
                    .take()
                    .expect("original completed task")
                    .into_retired()
                    .unwrap_or_else(|_| unreachable!("actual retirement reported complete")),
            );
        }
        // Old map, retired original data and grants stay in this SAME attempt.
        self.complete = true;
        Ok(())
    }
}

pub(crate) fn read_expected(
    source: &StoreColdSource,
    expected: AnchoredRecordVersion,
    request: &ExecutionRequest<'_, '_>,
) -> Result<AnchoredRecordReadMembership, Denial> {
    let records = source
        .serving()
        .anchored_records()
        .map_err(Denial::Unavailable)?;
    let reader = source
        .serving()
        .try_records_for_checkpoint(request)
        .map_err(|cause| match cause {
            Capture::Busy => Denial::Busy,
            Capture::Request(cause) => {
                Denial::Execution(worth_execution::WorkCeilingDenial::Stopped(cause))
            }
            Capture::Protection(cause) => Denial::Read(ReadFailure::Protection(cause)),
        })?;
    let mut stream = records
        .open_payload_from(&reader, request, expected.name(), expected.payload_bytes())
        .map_err(Denial::Read)?
        .ok_or(Denial::DifferentSelection)?;
    if stream.observed_generation() != expected.generation()
        || stream.payload_bytes() != expected.payload_bytes()
    {
        return Err(Denial::DifferentSelection);
    }
    let mut scratch = [0; 16 * 1024];
    while stream.read_next(&mut scratch).map_err(Denial::Read)? != 0 {}
    if stream.verified_version() != Some(expected) {
        return Err(Denial::DifferentSelection);
    }
    stream
        .into_verified_membership(reader)
        .map_err(|_| Denial::DifferentSelection)
}
fn try_read<T>(lock: &std::sync::RwLock<T>) -> Result<std::sync::RwLockReadGuard<'_, T>, Denial> {
    match lock.try_read() {
        Ok(guard) => Ok(guard),
        Err(TryLockError::Poisoned(error)) => Ok(error.into_inner()),
        Err(TryLockError::WouldBlock) => Err(Denial::Busy),
    }
}
fn try_write<T>(lock: &std::sync::RwLock<T>) -> Result<std::sync::RwLockWriteGuard<'_, T>, Denial> {
    match lock.try_write() {
        Ok(guard) => Ok(guard),
        Err(TryLockError::Poisoned(error)) => Ok(error.into_inner()),
        Err(TryLockError::WouldBlock) => Err(Denial::Busy),
    }
}

#[cfg(test)]
mod tests;
