//! Verified EOF transfers the ORIGINAL reader before another fallible consumer.
use super::*;
use worth_execution::{ExecutionImmutableBytes, ExecutionRequestArrayBuilder};

pub(super) fn selected(
    cut: &mut HomeReadCut,
    serving: &ServingPhysicalRuntime,
    request: &ExecutionRequest<'_, '_>,
) -> Result<Option<SelectedHomeManifest>, Denial> {
    request.consult().map_err(Denial::Execution)?;
    let records = serving.anchored_records().map_err(Denial::Unavailable)?;
    if cut.selected.is_none() {
        let reader = cut.reader.as_ref().ok_or(Denial::ReadSourceBusy)?;
        let anchor = AnchoredRecordName::new(crate::registry::CHECKPOINT_ANCHOR)
            .expect("closed Home anchor");
        let Some(mut stream) = records
            .open_payload_from(reader, request, anchor, 1 << 30)
            .map_err(anchor_denial)?
        else {
            // ONLY the actual SAME reader's anchor lookup can establish this.
            return Ok(None);
        };
        let size = usize::try_from(stream.payload_bytes()).map_err(|_| Denial::SizeLimit)?;
        let mut bytes =
            ExecutionRequestArrayBuilder::allocate(*request, size).map_err(Denial::Payload)?;
        let mut scratch = [0; 64 << 10];
        loop {
            let count = stream.read_next(&mut scratch).map_err(anchor_denial)?;
            if count == 0 {
                break;
            }
            bytes
                .extend_from_slice(&scratch[..count])
                .map_err(Denial::Payload)?;
        }
        let bytes = bytes
            .seal()
            .map_err(Denial::Payload)?
            .into_immutable_bytes();
        let reader = cut.reader.take().ok_or(Denial::ReadSourceBusy)?;
        let membership = match stream.into_verified_membership(reader) {
            Ok(membership) => membership,
            Err((_stream, reader)) => {
                cut.reader = Some(reader);
                return Err(Denial::ReadSourceBusy);
            }
        };
        cut.selected = Some(VerifiedHomeRead { membership, bytes });
    }
    let selected = cut
        .selected
        .as_ref()
        .expect("retained actual verified read");
    records
        .validate_read_membership(&selected.membership)
        .map_err(anchor_denial)?;
    SelectedHomeManifest::from_verified_read(selected.bytes.clone(), selected.membership.clone())
        .map(Some)
        .map_err(Denial::Manifest)
}

pub(super) fn journal_payload(
    cut: &HomeReadCut,
    serving: &ServingPhysicalRuntime,
    name: OrderedJournalName,
    position: u64,
    request: &ExecutionRequest<'_, '_>,
) -> Result<(OrderedJournalEntry, ExecutionImmutableBytes), Denial> {
    let journal = serving
        .ordered_journal(name)
        .map_err(Denial::JournalUnavailable)?;
    let mut stream = if let Some(selected) = &cut.selected {
        journal.open_payload_from_read(&selected.membership, request, position, 64 << 10)
    } else {
        journal.open_payload_from(
            cut.reader.as_ref().ok_or(Denial::ReadSourceBusy)?,
            request,
            position,
            64 << 10,
        )
    }
    .map_err(journal_denial)?;
    let entry = stream.entry();
    let size = usize::try_from(entry.payload_bytes()).map_err(|_| Denial::SizeLimit)?;
    let mut bytes =
        ExecutionRequestArrayBuilder::allocate(*request, size).map_err(Denial::Payload)?;
    let mut scratch = [0; 64 << 10];
    loop {
        let count = stream.read_next(&mut scratch).map_err(journal_denial)?;
        if count == 0 {
            break;
        }
        bytes
            .extend_from_slice(&scratch[..count])
            .map_err(Denial::Payload)?;
    }
    let bytes = bytes
        .seal()
        .map_err(Denial::Payload)?
        .into_immutable_bytes();
    Ok((entry, bytes))
}
fn anchor_denial(cause: AnchoredRecordReadFailure) -> Denial {
    match cause {
        AnchoredRecordReadFailure::Execution(cause) => Denial::Execution(cause),
        cause => Denial::Read(cause),
    }
}
fn journal_denial(cause: OrderedJournalReadFailure) -> Denial {
    match cause {
        OrderedJournalReadFailure::Execution(cause) => Denial::Execution(cause),
        cause => Denial::JournalRead(cause),
    }
}
