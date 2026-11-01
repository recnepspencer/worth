//! Original Native release descriptions share their actual admitted allocation.
use super::{WorldDurableFrameDenial as Frame, WorldDurableMediumDenial as Denial};
use std::sync::Arc;
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest};
use worth_relational::facade::history::{CommitId, RelationalCommitReceipt};

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
#[serde(transparent)]
pub(crate) struct WorldRelationalReleaseEvidence {
    meaning: Arc<ReleaseMeaning>,
}
#[derive(Debug, serde::Serialize)]
struct ReleaseMeaning {
    receipt: RelationalCommitReceipt,
    position: u64,
    digest: [u8; 32],
    #[serde(skip)]
    _funding: Option<ExecutionMemoryReservation>,
}
impl PartialEq for ReleaseMeaning {
    fn eq(&self, other: &Self) -> bool {
        self.receipt == other.receipt
            && self.position == other.position
            && self.digest == other.digest
    }
}
impl Eq for ReleaseMeaning {}
fn fixed_bytes() -> Result<u64, Denial> {
    std::mem::size_of::<ReleaseMeaning>()
        .checked_add(2 * std::mem::size_of::<usize>())
        .and_then(|bytes| u64::try_from(bytes).ok())
        .ok_or(Denial::Frame(Frame::SizeLimit))
}
impl WorldRelationalReleaseEvidence {
    pub(crate) fn from_original(
        execution: &ExecutionRequest<'_, '_>,
        receipt: &RelationalCommitReceipt,
        position: u64,
        digest: [u8; 32],
    ) -> Result<Self, Denial> {
        let fixed = fixed_bytes()?;
        let heap = receipt
            .parents
            .len()
            .checked_mul(std::mem::size_of::<CommitId>())
            .and_then(|bytes| bytes.checked_add(receipt.branch_id.0.len()))
            .and_then(|bytes| u64::try_from(bytes).ok())
            .and_then(|bytes| bytes.checked_add(fixed))
            .ok_or(Denial::Frame(Frame::SizeLimit))?;
        let funding = execution
            .reserve_retained_memory(heap)
            .map_err(Denial::Request)?;
        Ok(Self {
            meaning: Arc::new(ReleaseMeaning {
                receipt: receipt.clone(),
                position,
                digest,
                _funding: funding,
            }),
        })
    }
    pub(super) fn from_json_in(
        execution: &ExecutionRequest<'_, '_>,
        bytes: &[u8],
    ) -> Result<Self, Denial> {
        #[derive(serde::Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Fields<'a> {
            #[serde(borrow)]
            receipt: &'a serde_json::value::RawValue,
            position: u64,
            digest: [u8; 32],
        }
        let work = u64::try_from(bytes.len()).map_err(|_| Denial::Frame(Frame::SizeLimit))?;
        execution
            .run_native_with_checkpoint(|context| context.checkpoint(work))
            .map_err(Denial::Request)?
            .0
            .map_err(|cause| Denial::Request(worth_execution::WorkCeilingDenial::Stopped(cause)))?;
        let scratch = work
            .checked_mul(4)
            .and_then(|bytes| bytes.checked_add(256))
            .ok_or(Denial::Frame(Frame::SizeLimit))?;
        let _scratch = execution
            .reserve_retained_memory(scratch)
            .map_err(Denial::Request)?;
        let fields: Fields<'_> =
            serde_json::from_slice(bytes).map_err(|_| Denial::Frame(Frame::Malformed))?;
        let quote =
            RelationalCommitReceipt::recorded_json_heap_quote(fields.receipt.get().as_bytes())
                .map_err(|cause| Denial::NativeDescription(Box::new(cause)))?;
        let _growth = execution
            .reserve_retained_memory(quote.temporary_bytes())
            .map_err(Denial::Request)?;
        let heap = fixed_bytes()?
            .checked_add(quote.retained_bytes())
            .ok_or(Denial::Frame(Frame::SizeLimit))?;
        let funding = execution
            .reserve_retained_memory(heap)
            .map_err(Denial::Request)?;
        let receipt = serde_json::from_slice(fields.receipt.get().as_bytes())
            .map_err(|_| Denial::Frame(Frame::Malformed))?;
        execution.consult().map_err(Denial::Request)?;
        Ok(Self {
            meaning: Arc::new(ReleaseMeaning {
                receipt,
                position: fields.position,
                digest: fields.digest,
                _funding: funding,
            }),
        })
    }
    pub(crate) fn receipt(&self) -> &RelationalCommitReceipt {
        &self.meaning.receipt
    }
    pub(crate) fn position(&self) -> u64 {
        self.meaning.position
    }
    pub(crate) fn digest(&self) -> &[u8; 32] {
        &self.meaning.digest
    }
}
