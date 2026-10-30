pub(crate) use {crate::certification_suite::ordinary_context::certification_digest_assertions::assert_equal_on};
pub(crate) use {crate::certification_suite::ordinary_context::certification_worth_native_fixture::worth_native_state_projection_lanes::lower_direct_projection_bundle, crate::certification_suite::ordinary_context::certification_worth_native_fixture::worth_native_state_projection_lanes::lower_direct_state_bundle, crate::certification_suite::ordinary_context::certification_worth_native_fixture::worth_native_state_projection_lanes::product_projection_bundle, crate::certification_suite::ordinary_context::certification_worth_native_fixture::worth_native_state_projection_lanes::product_retained_bundle, crate::certification_suite::ordinary_context::certification_worth_native_fixture::worth_native_common::remask_server};

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
            crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationField::RequestContextDigest,
            crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationField::ResponseDigest,
            crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationField::ProvenanceDigest,
            crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationField::CounterSnapshot,
            crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationField::Output(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::SurfaceContract),
            crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationField::Output(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::Declaration),
            crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationField::Output(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::Handoff),
            crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationField::Output(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::SupportPosture),
            crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationField::Output(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::Branch),
            crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationField::Output(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::Workspace),
            crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationField::Output(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::RetainedState),
            crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationField::Output(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::Basis),
            crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationField::Output(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::Remask),
            crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationField::Output(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::AsyncResult),
            crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationField::Output(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::TemporalState),
        ],
    );
    assert_equal_on(
        &product_projection_lane,
        &lower_projection_lane,
        &[
            crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationField::RequestContextDigest,
            crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationField::ResponseDigest,
            crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationField::ProvenanceDigest,
            crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationField::CounterSnapshot,
            crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationField::Output(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::SurfaceContract),
            crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationField::Output(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::Declaration),
            crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationField::Output(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::Handoff),
            crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationField::Output(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::SupportPosture),
            crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationField::Output(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::Branch),
            crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationField::Output(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::Workspace),
            crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationField::Output(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::Basis),
            crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationField::Output(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::Remask),
            crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationField::Output(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::Policy),
            crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationField::Output(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::FactReceipt),
            crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationField::Output(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::Materialization),
            crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationField::Output(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::CounterSnapshot),
        ],
    );
}
