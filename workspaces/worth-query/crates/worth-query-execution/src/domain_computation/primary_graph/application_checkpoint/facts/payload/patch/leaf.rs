//! A copied leaf keeps raw copied Facts ahead of their funding on every exit.
use super::*;
use std::sync::OnceLock;
use worth_execution::ExecutionMemoryReservation;

pub(super) struct PreparedLeaf {
    pub(super) original: FactPayload,
    pub(super) changed: u32,
    pub(super) filled: u32,
    pub(super) replacements: ExecutionArray<RefCell<Option<Fact>>>,
    pub(super) output: ExecutionArrayBuilder<Fact>,
    pub(super) copied_heaps: Option<ExecutionMemoryReservation>,
    pub(super) replacement_heaps: ExecutionArray<OnceLock<ExecutionMemoryReservation>>,
}
impl PreparedLeaf {
    pub(super) fn finish(
        mut self,
        control: StorageControl<'_, '_>,
    ) -> Result<FactPayload, StoreDenial> {
        if self.changed != self.filled {
            return Err(StoreDenial::ConflictingBody);
        }
        for ordinal in 0..self.replacements.len() {
            control.check_live()?;
            let value = if self.changed & (1_u32 << ordinal) != 0 {
                self.replacements[ordinal]
                    .borrow_mut()
                    .take()
                    .ok_or(StoreDenial::ConflictingBody)?
            } else {
                // The exhaustive cached whole-leaf quote was reserved before
                // this first copy. All refusal paths drop output before funding.
                observation::native_patch_clone();
                self.original
                    .get(ordinal)
                    .expect("native leaf ordinal")
                    .clone()
            };
            self.output.push(value)?;
            observation::native_payload_fact_move();
        }
        let value = FactPayload::copied_native(
            self.output.seal()?,
            self.copied_heaps,
            Some(self.replacement_heaps),
        );
        observation::native_payload_chunk();
        Ok(value)
    }
}
