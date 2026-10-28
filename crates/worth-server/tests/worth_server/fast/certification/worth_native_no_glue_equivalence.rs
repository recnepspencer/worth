pub(crate) use super::certification_bundle::{
    WorthServerCertificationField as Field, WorthServerCertificationOutputDigest as Output,
};
pub(crate) use super::certification_digest_assertions::assert_equal_on;
pub(crate) use super::certification_worth_native_fixture::{
    compatibility_overlap_bundle, lower_direct_read_bundle, product_read_bundle, standard_server,
};

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
            Field::RequestContextDigest,
            Field::ResponseDigest,
            Field::ProvenanceDigest,
            Field::CounterSnapshot,
            Field::Output(Output::SurfaceContract),
            Field::Output(Output::Declaration),
            Field::Output(Output::DeclarationSupport),
            Field::Output(Output::Handoff),
            Field::Output(Output::SupportPosture),
            Field::Output(Output::Branch),
            Field::Output(Output::Workspace),
        ],
    );
    assert_equal_on(
        &product_lane,
        &compatibility_lane,
        &[
            Field::RequestContextDigest,
            Field::Output(Output::SurfaceContract),
            Field::Output(Output::Declaration),
            Field::Output(Output::DeclarationSupport),
            Field::Output(Output::Branch),
            Field::Output(Output::Workspace),
        ],
    );
}
