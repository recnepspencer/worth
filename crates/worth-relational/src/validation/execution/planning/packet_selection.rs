use crate::authority::commit::preparation::packets::invariant::InvariantPacketRegistration;
use crate::validation::data::CustomInvariantScopePlanner;
use crate::validation::engine::InvariantExecutionRequest;
use crate::validation::engine::InvariantRuntimeView;

pub(super) fn eligible_registrations<'runtime, 'state>(
    runtime: &'runtime InvariantRuntimeView,
    request: &'state InvariantExecutionRequest<'state>,
) -> Vec<InvariantPacketRegistration>
where
    'runtime: 'state,
{
    let native = runtime
        .config
        .schema
        .invariant_catalog
        .registrations_for_execution_point(request.execution_point())
        .chain(
            runtime
                .schema_contract_runtime
                .relation_integrity_registrations
                .iter()
                .filter(move |registration| {
                    registration.execution_point == request.execution_point()
                }),
        )
        .filter(|registration| request.includes_registration(registration))
        .cloned()
        .map(InvariantPacketRegistration::Native);

    let custom = runtime
        .schema_contract_runtime
        .custom_invariant_registries
        .iter()
        .filter(|registration| request.includes_custom_registration(registration))
        .map(|registration| {
            let work = crate::validation::custom_rule::CustomInvariantWorkMeter::new(
                registration.maximum_work_units(),
            );
            let prepared_scope = crate::validation::data::PreparedCustomInvariantScope::capture(
                request.observation(),
                request.version_id(),
                request.merged_plan(),
                &work,
            );
            let mut planner = CustomInvariantScopePlanner::new_at_current_version(
                runtime,
                request.observation(),
                request.version_id(),
                request.current_version_id(),
                &prepared_scope,
                work,
                std::sync::Arc::new(registration.access_contract().clone()),
            );
            let prepared_execution = registration
                .executable()
                .prepare_for_execution(runtime, &mut planner);
            InvariantPacketRegistration::Custom {
                registration: registration.clone(),
                prepared_execution,
                prepared_scope: prepared_scope.clone(),
            }
        });

    native.chain(custom).collect()
}
