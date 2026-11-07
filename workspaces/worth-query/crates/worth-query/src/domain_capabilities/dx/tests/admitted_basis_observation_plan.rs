use super::*;

pub(super) fn admitted_basis_observation_plan() -> crate::runtime::WorthQueryAdmittedIntentPlan {
    let request =
        crate::intent_admission::WorthQueryRawIntentAdmissionRequest::basis_observation_lane(
            crate::basis_lifecycle::RawBasisIntent::CurrentHead,
        )
        .expect("basis-observation request should build");

    match crate::intent_admission::admit_runtime_intent_request(request)
        .expect("fixture digest must fit")
    {
        crate::intent_admission::WorthQueryIntentAdmissionDecision::Admitted(plan) => plan,
        other => panic!("expected admitted basis-observation plan, got {other:?}"),
    }
}
