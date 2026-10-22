use super::WorthQueryValueDecodeDenial as Stop;
use worth_execution::MapKernelStop;
use worth_foundational::facade::{AspectValue, ValueCodecDenial};
use worth_query_installation::facade::{
    AdmittedFieldBindingStop, ApplicationReadableScalarValueBinding,
};

/// Every child read borrows the original control. Explicitly accounted reads
/// also retain their original meter and cumulative reservation.
pub(in crate::domain_computation::primary_graph) struct ValueDecodeSession<'a> {
    pub(super) state: &'a super::state::ValueDecodeState,
    pub(super) checkpoint: Option<&'a dyn Fn(u64) -> Result<(), MapKernelStop>>,
    pub(super) poll: &'a dyn Fn() -> Result<(), MapKernelStop>,
}

impl ValueDecodeSession<'_> {
    pub(in crate::domain_computation::primary_graph) fn stopped(&self) -> bool {
        self.state.denial().is_some()
    }

    pub(in crate::domain_computation::primary_graph) fn denial(&self) -> Option<Stop> {
        self.state.denial()
    }

    fn refuse(&self, stop: Stop) -> ValueCodecDenial {
        self.state.refuse(stop);
        ValueCodecDenial::Admission
    }

    fn admit(&self, work: usize, bytes: usize) -> Result<(), ValueCodecDenial> {
        if self.stopped() {
            return Err(ValueCodecDenial::Admission);
        }
        let Some(checkpoint) = self.checkpoint else {
            return self.poll_request();
        };
        let work = u64::try_from(work).map_err(|_| self.refuse(Stop::AccountingOverflow))?;
        checkpoint(work).map_err(|cause| self.refuse(Stop::Checkpoint(cause)))?;
        let mut memory = self.state.memory.borrow_mut();
        let held = memory
            .as_mut()
            .expect("decode scope owns its native reservation");
        let bytes = u64::try_from(bytes)
            .ok()
            .and_then(|bytes| held.bytes().checked_add(bytes))
            .ok_or_else(|| self.refuse(Stop::AccountingOverflow))?;
        held.resize(bytes)
            .map_err(|cause| self.refuse(Stop::Memory(cause)))
    }

    fn poll_request(&self) -> Result<(), ValueCodecDenial> {
        (self.poll)().map_err(|cause| self.refuse(Stop::Checkpoint(cause)))
    }

    pub(in crate::domain_computation::primary_graph) fn decode<
        Binding: ApplicationReadableScalarValueBinding,
    >(
        &self,
        entity: &str,
        aspect: &str,
        field: &str,
        raw: &AspectValue,
    ) -> Option<Binding::Value> {
        if self.stopped() {
            return None;
        }
        // Legacy scalar owners keep their existing contract. Canonical owners
        // cannot enter this arm: installation refuses missing admitted support.
        if !Binding::SUPPORTS_ADMITTED_DECODE {
            return Binding::decode(raw).ok();
        }
        let mut lookup = |work, bytes| {
            let work = usize::try_from(work).map_err(|_| self.refuse(Stop::AccountingOverflow))?;
            let bytes =
                usize::try_from(bytes).map_err(|_| self.refuse(Stop::AccountingOverflow))?;
            self.admit(work, bytes)
        };
        let binding = if self.checkpoint.is_some() {
            self.state
                .bindings
                .field_admitted(entity, aspect, field, &mut lookup)
        } else {
            self.poll_request().ok()?;
            Ok(self.state.bindings.field(entity, aspect, field))
        };
        let binding = match binding {
            Ok(Some(binding)) if binding.matches_binding::<Binding>() => binding,
            Err(AdmittedFieldBindingStop::Admission(_)) => return None,
            Err(AdmittedFieldBindingStop::AccountingOverflow) => {
                self.refuse(Stop::AccountingOverflow);
                return None;
            }
            _ => {
                self.refuse(Stop::BindingContractMismatch);
                return None;
            }
        };
        let value = binding
            .decode_admitted::<Binding::Value>(raw, &mut |work, bytes| self.admit(work, bytes));
        // A seed is permitted to return success after catching its callback's
        // error; that success never clears the original native refusal.
        if self.stopped() {
            return None;
        }
        match value {
            Ok(value) => Some(value),
            Err(_) => {
                self.refuse(Stop::CodecRejected);
                None
            }
        }
    }
}
