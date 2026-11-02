//! One branded body runner selects ordinary or Directory completion semantics.
use super::*;
impl QueryRequestExecution<'_> {
    pub(super) fn run_admitted_advancement<R>(
        &self,
        owner: Option<worth_runtime_world::facade::RuntimeWorldOwnerIdentity>,
        source_instance: Option<u64>,
        maintenance: Option<&dyn crate::domain_computation::execution_runtime::product_world::WorthQueryHomeMaintenance>,
        body: impl for<'scope> FnOnce(WorthQueryAdvancementPhase<'scope>) -> R,
    ) -> Result<R, WorthQueryAdvancementDenial> {
        let request = self
            .execution_request()
            .map_err(WorthQueryAdvancementDenial::Resource)?;
        if self.declared_work == Some(0) {
            return Err(WorthQueryAdvancementDenial::Resource(
                Resource::WorkExhausted,
            ));
        }
        // Keep the owning business result outside Execution's nested completion
        // frames. Those frames carry only completion, independent of R's size.
        let mut body_result = None;
        let mut body_unwind = None;
        let execute = |lease: Option<&worth_execution::ExecutionResourceLease<'_>>| {
            observation::record_workers(
                lease.map_or(1, |lease| lease.policy().max_workers().get()),
            );
            let active = ActiveAdvancement { execution: self };
            let source = match (source_instance, owner) {
                (Some(source), _) => phase_custody::AdvancementSource::Relational(source),
                (None, Some(owner)) => phase_custody::AdvancementSource::ProductOwner(owner),
                (None, None) => phase_custody::AdvancementSource::Bootstrap,
            };
            match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                let result = body(WorthQueryAdvancementPhase {
                    active: &active,
                    source,
                    owner,
                });
                if let Some(maintenance) = maintenance {
                    let invoking = lease.map(ExecutionRequest::leased).unwrap_or(request);
                    directory_completion::after_business(invoking, maintenance);
                }
                body_result = Some(result);
            })) {
                Ok(()) => (),
                Err(payload) => {
                    body_unwind = Some(payload);
                }
            }
        };
        let outcome = if self.has_accounting() {
            let ceiling = ExecutionWorkCeiling::new(self.declared_work);
            if maintenance.is_some() {
                request.run_completion(ceiling, execute)
            } else {
                request.run(ceiling, execute)
            }
        } else {
            let native = if maintenance.is_some() {
                request.run_native_completion(execute)
            } else {
                request.run_native(execute)
            };
            native.map(|(value, physical)| {
                let posture = if physical.active_workers_high_watermark() > 1 {
                    worth_foundational::ExecutionPosture::Automatic
                } else {
                    worth_foundational::ExecutionPosture::Serial
                };
                (
                    value,
                    worth_foundational::ExecutionReport::native(posture, physical),
                )
            })
        };
        // The execution owner catches unwinds to close its accounting scope.
        // Preserve the host body's original unwind and its existing posture.
        if let Some(payload) = body_unwind {
            observation::record_panicked(outcome.as_ref().ok().map(|(_, report)| *report));
            std::panic::resume_unwind(payload);
        }
        outcome
            .map(|((), report)| {
                observation::record_report(report);
                body_result.expect("a completed advancement has its body result")
            })
            .map_err(advancement_denial)
    }
}
