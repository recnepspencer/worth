use worth_query_declaration::{
    facade::{
        application_query::ApplicationQueryLiveCauseBinding,
        application_schema::{ApplicationEffectRef, ApplicationRetainedEffectBinding},
    },
    worth_query_effect,
};

use super::*;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct PlanningLiveEvent {
    account: u64,
    activity: u64,
}
worth_query_declaration::worth_query_portable_type!(
    PlanningLiveEvent => "worth.query.test.planning-live-event.v1"
);

worth_query_declaration::worth_query_structured_value_binding!(
    pub(super) PlanningLiveEventBinding for PlanningLiveEvent {
        identity: "worth.query.test.planning-live-event.v1"
    }
);

impl ApplicationRetainedEffectBinding for PlanningLiveEventBinding {
    fn retained_bytes(value: &Self::Value) -> u64 {
        u64::try_from(std::mem::size_of_val(value)).unwrap_or(u64::MAX)
    }
}

worth_query_effect!(
    pub(super) PlanningLiveEffect for PlanningTestSchema,
    payload PlanningLiveEventBinding
);

pub(super) struct PlanningLiveCause;
worth_query_declaration::worth_query_portable_type!(
    PlanningLiveCause => "worth.query.test.planning-live-cause.v1"
);

impl ApplicationQueryLiveCauseBinding<PlanningTestSchema, ActivityQuery, Account, Activity>
    for PlanningLiveCause
{
    type Effect = PlanningLiveEffect;
    type PayloadBinding = PlanningLiveEventBinding;
    type ScopeIdentity = u64;
    type TargetIdentity = u64;

    fn effect() -> ApplicationEffectRef<
        PlanningTestSchema,
        Self::Effect,
        <Self::PayloadBinding as worth_query_declaration::facade::application_schema::ApplicationStructuredValueBinding>::Value,
    >{
        PlanningLiveEffect::reference()
    }

    fn scope_identity(
        payload: &<Self::PayloadBinding as worth_query_declaration::facade::application_schema::ApplicationStructuredValueBinding>::Value,
    ) -> Self::ScopeIdentity {
        payload.account
    }

    fn target_identity(
        payload: &<Self::PayloadBinding as worth_query_declaration::facade::application_schema::ApplicationStructuredValueBinding>::Value,
    ) -> Self::TargetIdentity {
        payload.activity
    }
}

#[test]
fn live_lane_adds_only_its_declared_maintenance_requirement() {
    let query = installed_query();
    let parameters = admit_application_query_parameters(
        &query,
        ApplicationQueryParameterSet::new()
            .bind(account_parameter(), 7_u64)
            .unwrap(),
    )
    .unwrap();
    let one_shot = admitted_requirements(
        query.read_graph(),
        WorthQueryApplicationQueryLane::OneShot,
        32,
        parameters.identity(),
    );
    let live = admitted_requirements(
        query.read_graph(),
        WorthQueryApplicationQueryLane::Live,
        32,
        parameters.identity(),
    );

    assert!(
        !one_shot.contains_kind(&WorthQueryGraphReadAccessRequirementKind::LiveMaintenanceSupport)
    );
    assert!(live.contains_kind(&WorthQueryGraphReadAccessRequirementKind::LiveMaintenanceSupport));
    assert_eq!(live.rows().len(), one_shot.rows().len() + 1);
}
