use crate::authority::commit::preparation::packets::invariant::InvariantPacketRegistration;
use crate::validation::data::CustomInvariantScopePlanner;
use crate::validation::engine::InvariantExecutionRequest;
use crate::validation::engine::InvariantRuntimeView;
use std::mem::size_of;

use super::retained_bytes::{custom_access_retained_bytes, native_registration_retained_bytes};

pub(super) fn eligible_registrations<'state>(
    runtime: &InvariantRuntimeView<'state>,
    request: &'state InvariantExecutionRequest<'state>,
    preparation_budget: Option<&crate::validation::custom_rule::CustomPreparationBudget>,
) -> Vec<InvariantPacketRegistration> {
    let touched_kinds = std::cell::OnceCell::new();
    let native = runtime
        .config
        .schema
        .invariant_catalog
        .registrations
        .iter()
        .chain(
            runtime
                .schema_contract_runtime
                .relation_integrity_registrations
                .iter(),
        )
        .take_while(|_| preparation_budget.is_none_or(|budget| budget.try_plan_item(0)))
        .filter(|registration| {
            registration.execution_point == request.execution_point()
                && request.includes_registration(registration)
        })
        .filter_map(|registration| {
            if preparation_budget.is_some_and(|budget| {
                !budget.try_plan_item(
                    native_registration_retained_bytes(registration)
                        .saturating_add(2 * size_of::<InvariantPacketRegistration>() as u64),
                )
            }) {
                return None;
            }
            Some(InvariantPacketRegistration::Native(registration.clone()))
        });

    let custom = runtime
        .schema_contract_runtime
        .custom_invariant_registries
        .iter()
        .take_while(|_| preparation_budget.is_none_or(|budget| budget.try_plan_item(0)))
        .filter(|registration| request.includes_custom_registration(registration))
        .filter_map(|registration| {
            if preparation_budget.is_some_and(|budget| {
                !budget.try_plan_item(
                    (2 * size_of::<InvariantPacketRegistration>() as u64).saturating_add(
                        custom_access_retained_bytes(registration.access_contract()),
                    ),
                )
            }) {
                return None;
            }
            let mut work = crate::validation::custom_rule::CustomInvariantWorkMeter::new(
                registration.maximum_work_units(),
            );
            if let Some(budget) = preparation_budget {
                work = work.with_lease(budget);
            }
            let access = registration.access_contract();
            let has_declared_applicability = !access.affected_entity_kinds.is_empty()
                || !access.affected_relation_kinds.is_empty();
            if preparation_budget.is_none()
                && has_declared_applicability
                && touched_kinds
                    .get_or_init(|| {
                        super::custom_applicability::CandidateTouchedKinds::from_request(request)
                    })
                    .as_ref()
                    .is_some_and(|kinds| !kinds.may_affect(access))
            {
                return Some(InvariantPacketRegistration::CustomNotApplicable {
                    registration: registration.clone(),
                    prepared_scope: crate::validation::data::PreparedCustomInvariantScope::empty(),
                    work,
                });
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
            if preparation_budget.is_some_and(|budget| budget.stop().is_some()) {
                return None;
            }
            if request.merged_plan().is_some()
                && has_declared_applicability
                && !planner.has_applicable_touches()
                && !work.exceeded()
            {
                return Some(InvariantPacketRegistration::CustomNotApplicable {
                    registration: registration.clone(),
                    prepared_scope,
                    work,
                });
            }
            let prepared_execution = if let Some(budget) = preparation_budget {
                match registration.executable().prepare_for_execution_checked(
                    runtime,
                    &mut planner,
                    &work,
                ) {
                    Some(execution) => execution,
                    None => {
                        budget.deny_unchecked();
                        return Some(InvariantPacketRegistration::CustomNotApplicable {
                            registration: registration.clone(),
                            prepared_scope,
                            work,
                        });
                    }
                }
            } else {
                registration
                    .executable()
                    .prepare_for_execution(runtime, &mut planner)
            };
            if preparation_budget.is_some_and(|budget| budget.stop().is_some()) {
                return None;
            }
            let retained_touched = planner.retained_touched();
            Some(InvariantPacketRegistration::Custom {
                registration: registration.clone(),
                prepared_execution,
                prepared_scope: prepared_scope.clone(),
                retained_touched,
            })
        });

    native.chain(custom).collect()
}
