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
    let touched_kinds = std::cell::OnceCell::new();
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
            let access = registration.access_contract();
            let has_declared_applicability = !access.affected_entity_kinds.is_empty()
                || !access.affected_relation_kinds.is_empty();
            if has_declared_applicability
                && touched_kinds
                    .get_or_init(|| {
                        super::custom_applicability::CandidateTouchedKinds::from_request(request)
                    })
                    .as_ref()
                    .is_some_and(|kinds| !kinds.may_affect(access))
            {
                return InvariantPacketRegistration::CustomNotApplicable {
                    registration: registration.clone(),
                    prepared_scope: crate::validation::data::PreparedCustomInvariantScope::empty(),
                    work,
                };
            }
            let prepared_scope =
                crate::validation::data::PreparedCustomInvariantScope::capture_with_shared_inputs(
                    request.observation(),
                    request.version_id(),
                    request.merged_plan(),
                    registration.access_contract(),
                    &work,
                    runtime.shared_candidate_inputs(),
                );
            let mut planner = CustomInvariantScopePlanner::new_at_current_version(
                runtime,
                request.observation(),
                request.version_id(),
                request.current_version_id(),
                &prepared_scope,
                work.clone(),
                std::sync::Arc::new(registration.access_contract().clone()),
            );
            if request.merged_plan().is_some()
                && has_declared_applicability
                && !planner.has_applicable_touches()
                && !work.exceeded()
            {
                return InvariantPacketRegistration::CustomNotApplicable {
                    registration: registration.clone(),
                    prepared_scope,
                    work,
                };
            }
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
