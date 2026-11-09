//! Carried actual installed decoder, with original admission and sticky failure.
use super::*;
use std::sync::Arc;
use worth_query_installation::facade::WorthQueryInstalledApplicationValueBinding as Installed;

impl ValueDecodeSession<'_> {
    pub(in crate::domain_computation::primary_graph) fn shared_field(
        &self,
        entity: &str,
        aspect: &str,
        field: &str,
    ) -> Option<Arc<Installed>> {
        if self.stopped() {
            return None;
        }
        let result =
            self.state
                .bindings
                .shared_field_admitted(entity, aspect, field, &mut |work, bytes| {
                    let work =
                        usize::try_from(work).map_err(|_| self.refuse(Stop::AccountingOverflow))?;
                    let bytes = usize::try_from(bytes)
                        .map_err(|_| self.refuse(Stop::AccountingOverflow))?;
                    self.admit(work, bytes)
                });
        match result {
            Ok(Some(binding)) => Some(binding),
            Err(AdmittedFieldBindingStop::Admission(_)) => None,
            Err(AdmittedFieldBindingStop::AccountingOverflow) => {
                self.refuse(Stop::AccountingOverflow);
                None
            }
            _ => {
                self.refuse(Stop::BindingContractMismatch);
                None
            }
        }
    }

    pub(in crate::domain_computation::primary_graph) fn verify_binding<
        Binding: ApplicationReadableScalarValueBinding,
    >(
        &self,
        binding: &Installed,
    ) -> bool {
        if self.stopped() {
            return false;
        }
        if !Binding::SUPPORTS_ADMITTED_DECODE || !binding.matches_binding::<Binding>() {
            self.refuse(Stop::BindingContractMismatch);
            return false;
        }
        self.poll_request().is_ok()
    }

    pub(in crate::domain_computation::primary_graph) fn decode_installed<
        Binding: ApplicationReadableScalarValueBinding,
    >(
        &self,
        binding: &Installed,
        raw: &AspectValue,
    ) -> Option<Binding::Value> {
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

impl ValueDecodeSession<'_> {
    pub(in crate::domain_computation::primary_graph) fn accounting_overflow(&self) -> Stop {
        self.refuse(Stop::AccountingOverflow);
        Stop::AccountingOverflow
    }

    /// Fund write traversal and installed lookup using this exact request loan.
    pub(in crate::domain_computation::primary_graph) fn admit_write_quantities(
        &self,
        work: u64,
        bytes: u64,
    ) -> Result<(), Stop> {
        let work = usize::try_from(work).map_err(|_| {
            self.refuse(Stop::AccountingOverflow);
            Stop::AccountingOverflow
        })?;
        let bytes = usize::try_from(bytes).map_err(|_| {
            self.refuse(Stop::AccountingOverflow);
            Stop::AccountingOverflow
        })?;
        self.admit(work, bytes).map_err(|_| {
            self.denial()
                .expect("refused original admission is latched")
        })
    }
    pub(in crate::domain_computation::primary_graph) fn validate_domain_write(
        &self,
        binding: &Installed,
        value: &AspectValue,
    ) -> Result<(), worth_query_installation::facade::ApplicationValueDecodeDenial> {
        binding.validate_encoded_admitted(value, &mut |work, bytes| self.admit(work, bytes))
    }
}
