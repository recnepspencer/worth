//! One advancement lifecycle, independent of the caller's body and result types.
use super::{
    advancement_denial, observation,
    phase_custody::{ActiveAdvancement, AdvancementSource},
    Form, QueryRequestExecution, WorthQueryAdvancementDenial, WorthQueryAdvancementPhase,
};
use worth_execution::ExecutionWorkCeiling;

pub fn run_advancement(
    execution: &QueryRequestExecution<'_>,
    owner: Option<worth_runtime_world::facade::RuntimeWorldOwnerIdentity>,
    source_instance: Option<u64>,
    maintenance: Option<
        &dyn crate::domain_computation::execution_runtime::product_world::WorthQueryHomeMaintenance,
    >,
    body: &mut dyn FnMut(WorthQueryAdvancementPhase<'_>),
    discard: &mut dyn FnMut(),
) -> Result<(), WorthQueryAdvancementDenial> {
    let posture = match &execution.form {
        Form::Leased { lease, policy } | Form::Serial { lease, policy } => lease
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
    maintenance: Option<
        &dyn crate::domain_computation::execution_runtime::product_world::WorthQueryHomeMaintenance,
    >,
    body: &mut dyn FnMut(WorthQueryAdvancementPhase<'_>),
    discard: &mut dyn FnMut(),
) -> Result<(), WorthQueryAdvancementDenial> {
    // Follows the old owned FnOnce into Execution, then its successful result.
    // A native final refusal must destroy that result inside Execution's boundary.
    let handoff = AdvancementHandoff {
        discard: Some(discard),
    };
    let request = execution
        .execution_request()
        .map_err(WorthQueryAdvancementDenial::Resource)?;
    if execution.declared_work == Some(0) {
        return Err(WorthQueryAdvancementDenial::Resource(
            super::Resource::WorkExhausted,
        ));
    }
    let mut body_unwind = None;
    let execute = |lease: Option<&worth_execution::ExecutionResourceLease<'_>>| {
        observation::record_workers(lease.map_or(1, |lease| lease.policy().max_workers().get()));
        let active = ActiveAdvancement { execution };
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
    let outcome = if execution.has_accounting() {
        let ceiling = ExecutionWorkCeiling::new(execution.declared_work);
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
        .map(|(result, report)| {
            observation::record_report(report);
            result
                .expect("a completed advancement has its body result")
                .release();
        })
        .map_err(advancement_denial)
}

/// Owns only destruction at the original boundary, never request or phase authority.
struct AdvancementHandoff<'a> {
    discard: Option<&'a mut dyn FnMut()>,
}

impl AdvancementHandoff<'_> {
    fn release(mut self) {
        self.discard = None;
    }
}

impl Drop for AdvancementHandoff<'_> {
    fn drop(&mut self) {
        if let Some(discard) = self.discard.take() {
            discard();
        }
    }
}
