//! Ten-byte version eligibility from the original protected cut, before body bills.
use super::*;
pub(super) fn read(
    cut: &HomeReadCut,
    serving: &ServingPhysicalRuntime,
    name: OrderedJournalName,
    position: u64,
    request: &ExecutionRequest<'_, '_>,
) -> Result<[u8; 10], Denial> {
    request.consult().map_err(Denial::Execution)?;
    let journal = serving
        .ordered_journal(name)
        .map_err(Denial::JournalUnavailable)?;
    // This is a bounded prefix probe, never whole-payload admission or SHA proof.
    let mut stream = match &cut.selected {
        Some(selected) => {
            journal.open_payload_from_read(&selected.membership, request, position, u64::MAX)
        }
        None => journal.open_payload_from(
            cut.reader.as_ref().ok_or(Denial::ReadSourceBusy)?,
            request,
            position,
            u64::MAX,
        ),
    }
    .map_err(read::journal_denial)?;
    let mut prefix = [0; 10];
    let mut count = 0;
    while count < prefix.len() {
        let read = stream
            .read_next(&mut prefix[count..])
            .map_err(read::journal_denial)?;
        if read == 0 {
            return Err(Denial::JournalRead(OrderedJournalReadFailure::Damaged));
        }
        count += read;
    }
    Ok(prefix)
}
