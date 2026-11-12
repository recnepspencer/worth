//! Borrowed dictionary staging and final typed text rosters have distinct peaks.
use super::*;
use crate::symbols::data::{StoredSymbolText, Symbol, SymbolSnapshotWire};

pub(super) fn quote(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<OwnerAllocation, Refusal> {
    let mut bytes = OwnerAllocation::default();
    value.fields(request, |field, value| {
        if field != "entries" {
            return Ok(());
        }
        let count = value.elements(request, |value| {
            let mut index = 0;
            let count = value.elements(request, |value| {
                if index == 1 && !value.is_map() {
                    // System text remains System on the canonical wire; its
                    // final String copy is funded by the envelope owner.
                    add(&mut bytes, text(value, request)?)?;
                }
                index += 1;
                Ok(())
            })?;
            if count != 2 {
                return Err(malformed());
            }
            Ok(())
        })?;
        add(
            &mut bytes,
            vector_storage::<(Symbol, StoredSymbolText)>(count)?,
        )?;
        let scratch = std::alloc::Layout::array::<usize>(count).map_err(|_| malformed())?;
        bytes.transient = bytes
            .transient
            .checked_add(u64::try_from(scratch.size()).map_err(|_| malformed())?)
            .ok_or_else(malformed)?;

        let wire_backing = SymbolSnapshotWire::roster_storage_bytes(vector_capacity(count)?)
            .and_then(|bytes| u64::try_from(bytes).ok())
            .ok_or_else(malformed)?;
        // The parsed borrowed dictionary remains live during its conversion;
        // include both relocating wire backing and the final Vec growth.
        bytes.transient = bytes
            .transient
            .checked_add(wire_backing.checked_mul(2).ok_or_else(malformed)?)
            .and_then(|bytes| {
                bytes.checked_add(vector_storage::<(Symbol, StoredSymbolText)>(count).ok()?)
            })
            .ok_or_else(malformed)?;
        Ok(())
    })?;
    Ok(bytes)
}
