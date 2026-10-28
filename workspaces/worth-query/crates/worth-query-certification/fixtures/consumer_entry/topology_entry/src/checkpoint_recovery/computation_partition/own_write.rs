//! The own write a test arms into the region output's next decisions: each
//! armed decision writes an entry fact its own run gathered, read by the
//! decision first or written blind. The tests that arm it hold the checkpoint
//! recovery guard.

use std::sync::{Mutex, PoisonError};

use worth_query_host::facade::primary_graph::{
    HandlerExecutionDenial, WorthQueryInvariantMutationTarget,
};

use super::facts::{EntryNumber, EntryValueBits, SetEntry};
use super::region_output::Decision;
use super::*;

/// Whether an armed decision reads the value it writes before writing it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum OwnWriteRead {
    Observed,
    /// Only the computation's gather reads the value.
    Blind,
}

/// The value armed decisions write: the entry's number and the bits.
#[derive(Clone, Copy, Debug)]
pub(super) struct OwnWrite {
    pub(super) number: u64,
    pub(super) bits: u64,
    pub(super) read: OwnWriteRead,
}

/// What armed decisions write.
static OWN_WRITE: Mutex<Option<OwnWrite>> = Mutex::new(None);

/// Arms every next decision to make `write`, or disarms them.
pub(super) fn arm_own_write(write: Option<OwnWrite>) {
    *OWN_WRITE.lock().unwrap_or_else(PoisonError::into_inner) = write;
}

/// The entry an armed decision writes and the bits it writes; the decision
/// reads the entry's bits first when the armed write is observed.
pub(super) fn decide<Schema: TopologyModelBinding>(
    reader: &mut Decision<'_, '_, '_, Schema>,
) -> Result<
    Option<(WorthQueryInvariantMutationTarget<Schema, SetEntry>, u64)>,
    HandlerExecutionDenial,
> {
    let Some(write) = *OWN_WRITE.lock().unwrap_or_else(PoisonError::into_inner) else {
        return Ok(None);
    };
    let entry = reader.resolve_entity(EntryNumber::reference(), write.number)?;
    if write.read == OwnWriteRead::Observed {
        reader.field(&entry, EntryValueBits::reference())?;
    }
    Ok(Some((reader.mutation_target(&entry)?, write.bits)))
}
