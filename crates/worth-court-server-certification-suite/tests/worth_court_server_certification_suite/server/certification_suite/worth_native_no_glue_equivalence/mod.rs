pub(crate) use {crate::certification_suite::ordinary_context::certification_digest_assertions::assert_equal_on};
pub(crate) use {crate::certification_suite::ordinary_context::certification_worth_native_fixture::worth_native_read_lanes::compatibility_overlap_bundle, crate::certification_suite::ordinary_context::certification_worth_native_fixture::worth_native_read_lanes::lower_direct_read_bundle, crate::certification_suite::ordinary_context::certification_worth_native_fixture::worth_native_read_lanes::product_read_bundle, crate::certification_suite::ordinary_context::certification_worth_native_fixture::worth_native_common::standard_server};

#[test]
fn worth_native_no_glue_equivalence_preserves_shared_pipeline_truth_without_endpoint_residue() {
    let server = standard_server();
    let product_lane = product_read_bundle(&server, "users.profile");
    let lower_direct_lane = lower_direct_read_bundle(&server, "users.profile");
    let compatibility_lane = compatibility_overlap_bundle(&server, "users.profile");

    assert_equal_on(
        &product_lane,
        &lower_direct_lane,
        &[
            crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationField::RequestContextDigest,
            crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationField::ResponseDigest,
            crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationField::ProvenanceDigest,
            crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationField::CounterSnapshot,
            crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationField::Output(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::SurfaceContract),
            crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationField::Output(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::Declaration),
            crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationField::Output(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::DeclarationSupport),
            crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationField::Output(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::Handoff),
            crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationField::Output(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::SupportPosture),
            crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationField::Output(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::Branch),
            crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationField::Output(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::Workspace),
        ],
    );
    assert_equal_on(
        &product_lane,
        &compatibility_lane,
        &[
            crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationField::RequestContextDigest,
            crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationField::Output(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::SurfaceContract),
            crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationField::Output(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::Declaration),
            crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationField::Output(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::DeclarationSupport),
            crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationField::Output(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::Branch),
            crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationField::Output(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::Workspace),
        ],
    );
}
