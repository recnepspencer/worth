//! Verify each exact derived body once at the extent's actual selected pin.
use super::*;
use crate::domain_computation::primary_graph::derived_source::DerivedSourceObservation;
impl SourceReadVerification<'_> {
    #[expect(
        clippy::disallowed_methods,
        reason = "Addresses serve live allocation lookup and custody; canonical order uses declared identity"
    )]
    pub(in crate::domain_computation::primary_graph) fn check_derived(
        &mut self,
        body: &Arc<DerivedSourceObservation>,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        control: Option<&dyn Fn() -> Result<(), Cause>>,
    ) -> Result<bool, Failure> {
        let result = execution.run_native_with_checkpoint(|context| {
            DerivedSourceObservation::checkpoint(context).map_err(Failure::Execution)?;
            if let Some(control) = control {
                control().map_err(Failure::Execution)?;
            }
            let address = Arc::as_ptr(body) as usize;
            let equal = if let Some((original, equal)) = self.derived.get(&address) {
                if !Arc::ptr_eq(original, body) {
                    return Err(Failure::Unavailable);
                }
                *equal
            } else {
                let equal = match &self.pin {
                    image::VerificationImage::Committed(pin) => {
                        body.remains_equal_at(pin, execution, context, control)
                    }
                    image::VerificationImage::Applied { pin, .. } => body.remains_equal_on_pin(
                        pin,
                        self.pin.origin(),
                        execution,
                        context,
                        control,
                    ),
                }?;
                memory::grow_tree::<(usize, (Arc<DerivedSourceObservation>, bool))>(
                    &mut self._memory,
                    &mut self.bytes,
                    self.derived.len(),
                    execution,
                    context.has_accounting(),
                )
                .map_err(Failure::Execution)?;
                self.derived.insert(address, (Arc::clone(body), equal));
                equal
            };
            DerivedSourceObservation::checkpoint(context).map_err(Failure::Execution)?;
            if let Some(control) = control {
                control().map_err(Failure::Execution)?;
            }
            Ok(equal)
        });
        match result {
            Ok((value, _)) => value,
            Err(stop) => Err(Failure::Execution(stop.into())),
        }
    }
}
