//! First controlled selected-member admission retains the actual physical cut.
use super::selection::{self, RelationalStoreMemberSelectionDenial as Denial, SelectedMembers};
use super::StoreColdSource;
use std::sync::{RwLockReadGuard, RwLockWriteGuard, TryLockError};
use worth_execution::ExecutionRequest;
use worth_store::physical_runtime::{
    AnchoredRecordName, AnchoredRecordReadMembership, AnchoredRecordVersion,
};

#[cfg(test)]
mod tests;

impl StoreColdSource {
    /// Install the original selected read once. An exact CONTROLLED retry is
    /// allocation-free; an ordinary same-version map is not such a retry.
    pub(crate) fn select_initial_members_from_read(
        &self,
        selected: &AnchoredRecordReadMembership,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), Denial> {
        let records = self
            .serving()
            .anchored_records()
            .map_err(Denial::Unavailable)?;
        let membership = records
            .validate_read_membership(selected)
            .map_err(Denial::Read)?;
        let version = membership.version();
        if version.name() != AnchoredRecordName::new("home.checkpoint").expect("static anchor") {
            return Err(Denial::ForeignAnchor);
        }
        request.check_live().map_err(|cause| {
            Denial::Execution(worth_execution::WorkCeilingDenial::Stopped(cause))
        })?;
        if self
            .custody
            .try_select(|| {
                let current = try_read(&self.members)?;
                initial_state(&current, version)
            })
            .map_err(|_| Denial::Busy)??
        {
            return Ok(());
        }
        let mut built = selection::build(membership.members(), |map, key, value| {
            map.insert_cloned_native_for_request(key, value, *request)
        })?;
        self.install_initial_built(selected, &mut built, request)
    }

    /// Reuse the SAME verified read and its actual prepared map across ACK
    /// Busy/stop. All original initial-state checks remain the one authority.
    pub(in crate::cold_source) fn install_initial_built(
        &self,
        selected: &AnchoredRecordReadMembership,
        built: &mut selection::MemberMap,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), Denial> {
        let records = self
            .serving()
            .anchored_records()
            .map_err(Denial::Unavailable)?;
        let membership = records
            .validate_read_membership(selected)
            .map_err(Denial::Read)?;
        let version = membership.version();
        if version.name() != AnchoredRecordName::new("home.checkpoint").expect("static anchor") {
            return Err(Denial::ForeignAnchor);
        }
        // This private handoff accepts only the complete actual table image,
        // including duplicate-content rows mapped to one real listed index.
        for member in membership.members() {
            request.consult().map_err(Denial::Execution)?;
            let key = worth_relational::facade::durability::RelationalDurableMemberLocator {
                sha256: member.sha256(),
                length: member.length(),
            };
            if !built.contains_key(&key) {
                return Err(Denial::DifferentSelection);
            }
        }
        for (key, placed) in built.iter() {
            request.consult().map_err(Denial::Execution)?;
            if membership.members().get(placed.index) != Some(&placed.locator)
                || placed.locator.sha256() != key.sha256
                || placed.locator.length() != key.length
            {
                return Err(Denial::DifferentSelection);
            }
        }
        // No I/O, allocation, physical-share loss, or nonempty old-map disposal
        // occurs under custody -> placement. On refusal `built` drops outside.
        self.custody
            .try_select(|| {
                let mut current = try_write(&self.members)?;
                let retry = initial_state(&current, version)?;
                request.check_live().map_err(|cause| {
                    Denial::Execution(worth_execution::WorkCeilingDenial::Stopped(cause))
                })?;
                if !retry {
                    std::mem::swap(&mut current.entries, built);
                    current.read = Some(selected.clone());
                    current.version = Some(version);
                }
                Ok(())
            })
            .map_err(|_| Denial::Busy)?
    }
}

fn initial_state(
    current: &SelectedMembers,
    version: AnchoredRecordVersion,
) -> Result<bool, Denial> {
    if current.version == Some(version) && current.read.is_some() {
        return Ok(true);
    }
    if current.version.is_none() && current.entries.is_empty() && current.read.is_none() {
        return Ok(false);
    }
    Err(Denial::DifferentSelection)
}
fn try_read<T>(lock: &std::sync::RwLock<T>) -> Result<RwLockReadGuard<'_, T>, Denial> {
    match lock.try_read() {
        Ok(guard) => Ok(guard),
        Err(TryLockError::Poisoned(error)) => Ok(error.into_inner()),
        Err(TryLockError::WouldBlock) => Err(Denial::Busy),
    }
}
fn try_write<T>(lock: &std::sync::RwLock<T>) -> Result<RwLockWriteGuard<'_, T>, Denial> {
    match lock.try_write() {
        Ok(guard) => Ok(guard),
        Err(TryLockError::Poisoned(error)) => Ok(error.into_inner()),
        Err(TryLockError::WouldBlock) => Err(Denial::Busy),
    }
}
