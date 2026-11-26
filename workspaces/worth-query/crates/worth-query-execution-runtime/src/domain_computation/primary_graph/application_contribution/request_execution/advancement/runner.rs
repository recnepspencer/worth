//! One advancement lifecycle, independent of the caller's body and result types.
use super::{
    advancement_denial, observation,
    phase_custody::{ActiveAdvancement, AdvancementSource},
    Form, QueryRequestExecution, WorthQueryAdvancementDenial, WorthQueryAdvancementPhase,
};
use crate::domain_computation::execution_runtime::product_world::published_effect::PublishedEffect;
use worth_execution::ExecutionWorkCeiling;

pub fn run_advancement(
    execution: &QueryRequestExecution<'_>,
    owner: Option<worth_runtime_world::facade::RuntimeWorldOwnerIdentity>,
    source_instance: Option<u64>,
    maintenance: Option<&dyn crate::ports::HomeMaintenanceAfterAdvancement>,
    retain_returned_outcome: bool,
    body: &mut dyn FnMut(WorthQueryAdvancementPhase<'_>),
    discard: &mut dyn FnMut(),
) -> Result<(), WorthQueryAdvancementDenial> {
    let posture = match &execution.form {
        Form::Leased { lease, policy, .. } | Form::Serial { lease, policy, .. } => lease
            .as_ref()
            .map(|lease| lease.resolved_posture())
            .unwrap_or(policy.posture()),
    };
    observation::begin(posture, execution.has_accounting());
    let result = run_admitted_advancement(
        execution,
        owner,
        source_instance,
        maintenance,
        retain_returned_outcome,
        body,
        discard,
    );
    observation::record_result(&result);
    result
}

fn run_admitted_advancement(
    execution: &QueryRequestExecution<'_>,
    owner: Option<worth_runtime_world::facade::RuntimeWorldOwnerIdentity>,
    source_instance: Option<u64>,
    maintenance: Option<&dyn crate::ports::HomeMaintenanceAfterAdvancement>,
    retain_returned_outcome: bool,
    body: &mut dyn FnMut(WorthQueryAdvancementPhase<'_>),
    discard: &mut dyn FnMut(),
) -> Result<(), WorthQueryAdvancementDenial> {
    // Follows the old owned FnOnce into Execution, then its successful result.
    // Ordinary native advancement destroys an unobserved result at a late
    // refusal. An effectful completion instead transfers the callback's exact
    // returned owner, including refusals, through Execution's completion primitive.
    // World's publication marker still protects ordinary callers (public #132).
    let published = PublishedEffect::fresh();
    let _published_scope = PublishedEffect::install(Some(published.clone()));
    let handoff = AdvancementHandoff {
        discard: Some(discard),
        published: published.clone(),
    };
    let caller_serial = match &execution.form {
        Form::Leased { caller_serial, .. } | Form::Serial { caller_serial, .. } => *caller_serial,
    };
    if let Some(caller) = caller_serial
        && let Err(stop) = caller.check_live()
    {
        return Err(advancement_denial(
            worth_execution::WorkCeilingDenial::Stopped(stop),
        ));
    }
    if execution.declared_work == Some(0) {
        return Err(WorthQueryAdvancementDenial::Resource(
            super::Resource::WorkExhausted,
        ));
    }
    let mut body_unwind = None;
    let run_inner = |active_execution: &QueryRequestExecution<'_>| {
        let request = match active_execution.execution_request() {
            Ok(request) => request,
            Err(_) => {
                // Re-extract the original LeaseDenial through `execution_request`.
                return Err(match &active_execution.form {
                    Form::Leased {
                        lease: Err(denial), ..
                    }
                    | Form::Serial {
                        lease: Err(denial), ..
                    } => worth_execution::WorkCeilingDenial::Admission(*denial),
                    _ => worth_execution::WorkCeilingDenial::Stopped(
                        worth_execution::MapKernelStop::AccountingUnavailable,
                    ),
                });
            }
        };
        let execute = |lease: Option<&worth_execution::ExecutionResourceLease<'_>>| {
            observation::record_workers(
                lease.map_or(1, |lease| lease.policy().max_workers().get()),
            );
            let active = ActiveAdvancement {
                execution: active_execution,
                admitted_at: std::sync::OnceLock::new(),
            };
            let source = match (source_instance, owner) {
                (Some(source), _) => AdvancementSource::Relational(source),
                (None, Some(owner)) => AdvancementSource::ProductOwner(owner),
                (None, None) => AdvancementSource::Bootstrap,
            };
            match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                body(WorthQueryAdvancementPhase {
                    active: &active,
                    source,
                    owner,
                });
                if let Some(maintenance) = maintenance {
                    let invoking = lease
                        .map(worth_execution::ExecutionRequest::leased)
                        .unwrap_or(request);
                    super::directory_completion::after_business(invoking, maintenance);
                }
            })) {
                Ok(()) => Some(handoff),
                Err(payload) => {
                    body_unwind = Some(payload);
                    None
                }
            }
        };
        if active_execution.has_accounting() {
            let ceiling = ExecutionWorkCeiling::new(active_execution.declared_work);
            if retain_returned_outcome || maintenance.is_some() {
                request.run_completion(ceiling, execute)
            } else {
                request.run(ceiling, execute)
            }
        } else {
            let native = if retain_returned_outcome || maintenance.is_some() {
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
        }
    };
    let outcome = match caller_serial {
        Some(caller) => {
            let run_within_serial = |_: Option<&worth_execution::ExecutionResourceLease<'_>>| {
                let scoped = execution.within_active_serial_request(caller);
                run_inner(&scoped)
            };
            let outer = if caller.has_accounting() || caller.work_ceiling().is_some() {
                caller
                    .run_completion(
                        ExecutionWorkCeiling::new(caller.work_ceiling()),
                        run_within_serial,
                    )
                    .map(|(inner, _)| inner)
            } else {
                caller
                    .run_native_completion(run_within_serial)
                    .map(|(inner, _)| inner)
            };
            match outer {
                Ok(inner) => inner,
                Err(denial) => Err(denial),
            }
        }
        None => run_inner(execution),
    };
    // The execution owner catches unwinds to close its accounting scope.
    // Preserve the host body's original unwind and its existing posture.
    if let Some(payload) = body_unwind {
        observation::record_panicked(outcome.as_ref().ok().map(|(_, report)| *report));
        std::panic::resume_unwind(payload);
    }
    match outcome {
        Ok((result, report)) => {
            observation::record_report(report);
            result
                .expect("a completed advancement has its body result")
                .release();
            Ok(())
        }
        // The only reader of the marker. World's outcome set it, so the body
        // published and completed (a body that did not complete unwound above);
        // the handoff kept its result. Only the native handoff refuses after
        // the body, so this arm never applies to the accounted path.
        Err(worth_execution::WorkCeilingDenial::Stopped(_)) if published.observed() => {
            observation::record_kept_after_publication();
            Ok(())
        }
        Err(denial) => Err(advancement_denial(denial)),
    }
}

/// Owns only destruction at the original boundary, never request or phase authority.
struct AdvancementHandoff<'a> {
    discard: Option<&'a mut dyn FnMut()>,
    /// Set from World's publication outcome; a published body's result is kept.
    published: PublishedEffect,
}

impl AdvancementHandoff<'_> {
    fn release(mut self) {
        self.discard = None;
    }
}

impl Drop for AdvancementHandoff<'_> {
    fn drop(&mut self) {
        if self.published.observed() {
            return;
        }
        if let Some(discard) = self.discard.take() {
            discard();
        }
    }
}
