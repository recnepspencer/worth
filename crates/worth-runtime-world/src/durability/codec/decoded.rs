//! The exact decoded World strings and record share one admitted cold allocation.
use super::*;
use std::sync::Arc;

#[derive(Debug)]
pub(crate) struct DecodedWorldRecord {
    pub(crate) record: WorldRecord,
    _funding: Option<worth_execution::ExecutionMemoryReservation>,
}
pub(in crate::durability) fn admit(
    bytes: &[u8],
    home: HomeIdentity,
    execution: &ExecutionRequest<'_, '_>,
) -> Result<Arc<DecodedWorldRecord>, WorldDurableMediumDenial> {
    use WorldDurableMediumDenial as Medium;
    let work = u64::try_from(bytes.len()).map_err(|_| Medium::Frame(Denial::SizeLimit))?;
    execution
        .run_native_with_checkpoint(|context| context.checkpoint(work))
        .map_err(Medium::Request)?
        .0
        .map_err(|cause| Medium::Request(worth_execution::WorkCeilingDenial::Stopped(cause)))?;
    let mut read = Read::new(bytes, home);
    let tag = read.fixed::<1>()?[0];
    let strings = match tag {
        1 => 0,
        2 | 5 | 6 | 9 => read.branch_name_bytes()?,
        7 | 8 => super::creation_name_extent::read(&mut read)?,
        3 | 4 => {
            let association = read.branch_name_bytes()?;
            // attempt/incarnation, then reserved commit/custody.
            for _ in 0..4 {
                read.number()?;
            }
            association
                .checked_add(read.branch_name_bytes()?)
                .ok_or(Denial::SizeLimit)?
        }
        _ => return Err(Denial::Malformed.into()),
    };
    let quote = std::mem::size_of::<DecodedWorldRecord>()
        .checked_add(2 * std::mem::size_of::<usize>())
        .and_then(|bytes| bytes.checked_add(strings))
        .and_then(|bytes| u64::try_from(bytes).ok())
        .ok_or(Denial::SizeLimit)?;
    let funding = execution
        .reserve_retained_memory(quote)
        .map_err(Medium::Request)?;
    let record = super::decode_record(bytes, home, execution)?;
    execution.consult().map_err(Medium::Request)?;
    Ok(Arc::new(DecodedWorldRecord {
        record,
        _funding: funding,
    }))
}
