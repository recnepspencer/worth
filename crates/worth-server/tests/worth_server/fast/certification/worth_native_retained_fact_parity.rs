pub(crate) use super::certification_bundle::{
    WorthServerCertificationField as Field, WorthServerCertificationOutputDigest as Output,
};
pub(crate) use super::certification_digest_assertions::assert_equal_on;
pub(crate) use super::certification_worth_native_fixture::{
    lower_direct_projection_bundle, lower_direct_state_bundle, product_projection_bundle,
    product_retained_bundle, remask_server,
};

#[test]
fn worth_native_retained_posture_and_typed_fact_certification_stay_parity_safe() {
    let server = remask_server();
    let product_retained_lane = product_retained_bundle(&server, "users.profile");
    let lower_state_lane = lower_direct_state_bundle(&server, "users.profile");
    let product_projection_lane = product_projection_bundle(&server, "users.profile");
    let lower_projection_lane = lower_direct_projection_bundle(&server, "users.profile");

    assert_equal_on(
        &product_retained_lane,
        &lower_state_lane,
        &[
            Field::RequestContextDigest,
            Field::ResponseDigest,
            Field::ProvenanceDigest,
            Field::CounterSnapshot,
            Field::Output(Output::SurfaceContract),
            Field::Output(Output::Declaration),
            Field::Output(Output::Handoff),
            Field::Output(Output::SupportPosture),
            Field::Output(Output::Branch),
            Field::Output(Output::Workspace),
            Field::Output(Output::RetainedState),
            Field::Output(Output::Basis),
            Field::Output(Output::Remask),
            Field::Output(Output::AsyncResult),
            Field::Output(Output::TemporalState),
        ],
    );
    assert_equal_on(
        &product_projection_lane,
        &lower_projection_lane,
        &[
            Field::RequestContextDigest,
            Field::ResponseDigest,
            Field::ProvenanceDigest,
            Field::CounterSnapshot,
            Field::Output(Output::SurfaceContract),
            Field::Output(Output::Declaration),
            Field::Output(Output::Handoff),
            Field::Output(Output::SupportPosture),
            Field::Output(Output::Branch),
            Field::Output(Output::Workspace),
            Field::Output(Output::Basis),
            Field::Output(Output::Remask),
            Field::Output(Output::Policy),
            Field::Output(Output::FactReceipt),
            Field::Output(Output::Materialization),
            Field::Output(Output::CounterSnapshot),
        ],
    );
}
