//! A corrupted comparison witness never becomes stored admitted custody.
use super::*;
impl RelationalContentFrame {
    pub(in crate::content) fn duplicate_comparison(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        format: &Format,
    ) -> Result<Self, Denial> {
        if !super::super::admission_fault::consume(format) {
            return Ok(self.clone());
        }
        let mut comparison = Self::readmit(execution, format, self.identity(), self.bytes())?;
        let admitted = Arc::get_mut(&mut comparison.0).ok_or(Denial::EncodingInconsistency)?;
        let backing =
            Arc::get_mut(&mut admitted.record.backing).ok_or(Denial::EncodingInconsistency)?;
        let payload = backing
            .bytes
            .get_mut(admitted.payload_offset..)
            .ok_or(Denial::EncodingInconsistency)?;
        let last = payload.last_mut().ok_or(Denial::EncodingInconsistency)?;
        *last ^= 1;
        Ok(comparison)
    }
}
