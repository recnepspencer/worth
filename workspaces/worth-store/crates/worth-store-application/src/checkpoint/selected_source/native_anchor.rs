//! Initialization is read under the SAME original Home cut as both native tails.
use super::*;
use worth_execution::{ExecutionImmutableBytes, ExecutionRequestArrayBuilder};
pub(super) fn read(
    cut: &HomeReadCut,
    serving: &ServingPhysicalRuntime,
    request: &ExecutionRequest<'_, '_>,
) -> Result<Option<ExecutionImmutableBytes>, Denial> {
    let records = serving.anchored_records().map_err(Denial::Unavailable)?;
    let name = AnchoredRecordName::new(crate::registry::RELATIONAL_INITIAL_INPUTS_ANCHOR)
        .expect("closed Native initialization anchor");
    let stream = match &cut.selected {
        Some(selected) => {
            records.open_payload_from_read(&selected.membership, request, name, 1 << 30)
        }
        None => records.open_payload_from(
            cut.reader.as_ref().ok_or(Denial::ReadSourceBusy)?,
            request,
            name,
            1 << 30,
        ),
    }
    .map_err(read::anchor_denial)?;
    let Some(mut stream) = stream else {
        return Ok(None);
    };
    let size = usize::try_from(stream.payload_bytes()).map_err(|_| Denial::SizeLimit)?;
    let mut bytes =
        ExecutionRequestArrayBuilder::allocate(*request, size).map_err(Denial::Payload)?;
    let mut scratch = [0; 16 * 1024];
    loop {
        let count = stream
            .read_next(&mut scratch)
            .map_err(read::anchor_denial)?;
        if count == 0 {
            break;
        }
        if count > size - bytes.len() {
            return Err(Denial::SizeLimit);
        }
        bytes
            .extend_from_slice(&scratch[..count])
            .map_err(Denial::Payload)?;
    }
    if bytes.len() != size || stream.verified_version().is_none() {
        return Err(Denial::Read(AnchoredRecordReadFailure::Damaged(
            AnchoredRecordDamage::Truncated,
        )));
    }
    Ok(Some(bytes.seal_immutable_bytes().map_err(Denial::Payload)?))
}
