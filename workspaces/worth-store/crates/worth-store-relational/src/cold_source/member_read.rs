//! Member reads follow the original physical selection when one is retained.
use super::{record_failure, Cause, RelationalDurableMemberLocator, StoreColdSource};
use std::sync::atomic::Ordering;
use worth_store::physical_runtime::{
    AnchoredRecordMemberDamage, AnchoredRecordMemberReadFailure as Failure,
    AnchoredRecordMemberStream, AnchoredRecordName,
};

impl StoreColdSource {
    pub(super) fn read_selected_member(
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
            .serving()
            .anchored_records()
            .map_err(|_| Cause::SourceUnavailable)?;
        let (placed, selected_read) = {
            let selected = self
                .members
                .read()
                .unwrap_or_else(|error| error.into_inner());
            let placed = selected
                .entries
                .get(&locator)
                .copied()
                .ok_or(Cause::MissingMember)?;
            (placed, selected.read.clone())
        };
        // Snapshot custody before I/O; no placement guard crosses a physical read.
        match selected_read {
            Some(selected) => {
                let stream = records
                    .open_read_member(&selected, placed.index, limit)
                    .map_err(|failure| member_failure(failure, limit))?;
                collect_member(stream, locator.length, limit)
            }
            None => records
                .read_member(
                    AnchoredRecordName::new("home.checkpoint").expect("static anchor"),
                    placed.locator,
                    limit,
                )
                .map_err(|failure| member_failure(failure, limit)),
        }
    }
}

/// The byte-returning legacy port keeps its existing unbudgeted allocation
/// contract. Controlled payload funding is a separate Native read obligation.
fn collect_member(
    mut stream: AnchoredRecordMemberStream,
    expected: u64,
    limit: u64,
) -> Result<Vec<u8>, Cause> {
    let length = usize::try_from(expected).map_err(|_| Cause::MemoryExhausted)?;
    let mut bytes = Vec::new();
    bytes
        .try_reserve_exact(length)
        .map_err(|_| Cause::MemoryExhausted)?;
    let mut scratch = [0; 16 << 10];
    loop {
        let count = stream
            .read_next(&mut scratch)
            .map_err(|failure| member_failure(failure, limit))?;
        if count == 0 {
            return if bytes.len() == length {
                Ok(bytes)
            } else {
                Err(Cause::Damaged)
            };
        }
        if count > length - bytes.len() {
            return Err(Cause::Damaged);
        }
        bytes.extend_from_slice(&scratch[..count]);
    }
}

pub(super) fn member_failure(failure: Failure, limit: u64) -> Cause {
    match failure {
        Failure::NotCurrentMember => Cause::MissingMember,
        Failure::LimitExceeded { length } => Cause::LimitExceeded { length, limit },
        Failure::Damaged(AnchoredRecordMemberDamage::DigestMismatch) => Cause::IntegrityMismatch,
        Failure::Damaged(_) => Cause::Damaged,
        // Failure to verify the selecting anchor proves nothing about this
        // member's bytes; it cannot poison every region cell.
        Failure::Value(_) => Cause::SourceUnavailable,
        Failure::Open(error) => record_failure(error),
        _ => Cause::SourceUnavailable,
    }
}
