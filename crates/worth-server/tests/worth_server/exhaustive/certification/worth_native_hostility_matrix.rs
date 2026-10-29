pub(crate) use super::certification_bundle::{
    WorthServerCertificationField as Field, WorthServerCertificationOutputDigest as Output,
};
pub(crate) use super::certification_counter_assertions::assert_counter_exact;
pub(crate) use super::certification_digest_assertions::{assert_equal_on, assert_not_equal_on};
pub(crate) use super::certification_worth_native_fixture::{
    branch_product_read_bundle, compatibility_durable_delivery_denial_bundle,
    compatibility_overlap_bundle, compatibility_runtime_backed_delivery_bundle,
    durable_delivery_denial_bundle, durable_later_server, forensic_server,
    lower_direct_read_bundle, product_projection_bundle, product_read_bundle, remask_server,
    retained_artifact_denial_bundle, runtime_backed_delivery_bundle, runtime_backed_server,
    standard_server, view_shape_product_read_bundle,
};
pub(crate) use worth_server::WorthServerDirectViewShape;

#[test]
fn worth_native_hostility_matrix_preserves_canonical_direct_digests_across_pressure_lanes() {
    let standard_server = standard_server();
    let forensic_server = forensic_server();
    let remask_server = remask_server();
    let runtime_backed_server = runtime_backed_server();
    let durable_later_server = durable_later_server();

    let control_lane = product_read_bundle(&standard_server, "users.profile");
    let equivalent_lane = lower_direct_read_bundle(&standard_server, "users.profile");
    let compatibility_lane = compatibility_overlap_bundle(&standard_server, "users.profile");
    let forensic_lane = product_read_bundle(&forensic_server, "users.profile");
    let branch_lane = branch_product_read_bundle(&standard_server, "users.profile", "branch-9");
    let detail_lane = view_shape_product_read_bundle(
        &standard_server,
        "users.profile",
        WorthServerDirectViewShape::Detail,
    );
    let table_lane = view_shape_product_read_bundle(
        &standard_server,
        "users.profile",
        WorthServerDirectViewShape::Table,
    );
    let runtime_backed_lane =
        runtime_backed_delivery_bundle(&runtime_backed_server, "users.profile");
    let compatibility_runtime_backed_lane =
        compatibility_runtime_backed_delivery_bundle(&runtime_backed_server, "users.profile");
    let durable_later_lane = durable_delivery_denial_bundle(&durable_later_server, "users.profile");
    let compatibility_durable_later_lane =
        compatibility_durable_delivery_denial_bundle(&durable_later_server, "users.profile");
    let remask_lane = product_projection_bundle(&remask_server, "users.profile");
    let retained_denial_lane =
        retained_artifact_denial_bundle(&standard_server, "users.profile.missing");

    assert_equal_on(
        &control_lane,
        &equivalent_lane,
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
            Field::Output(Output::SupportMatrix),
            Field::Output(Output::ViewShape),
        ],
    );
    assert_equal_on(
        &control_lane,
        &compatibility_lane,
        &[
            Field::Output(Output::SurfaceContract),
            Field::Output(Output::Declaration),
            Field::Output(Output::DeclarationSupport),
            Field::Output(Output::Branch),
            Field::Output(Output::Workspace),
        ],
    );
    assert_equal_on(
        &control_lane,
        &forensic_lane,
        &[
            Field::Output(Output::SurfaceContract),
            Field::Output(Output::Declaration),
            Field::Output(Output::Branch),
            Field::Output(Output::Workspace),
            Field::Output(Output::SupportMatrix),
            Field::Output(Output::ViewShape),
        ],
    );
    assert_not_equal_on(
        &control_lane,
        &forensic_lane,
        &[Field::RequestContextDigest],
    );
    assert_not_equal_on(
        &control_lane,
        &compatibility_lane,
        &[Field::Output(Output::SupportPosture)],
    );
    assert_not_equal_on(
        &control_lane,
        &branch_lane,
        &[Field::RequestContextDigest, Field::Output(Output::Branch)],
    );
    assert_equal_on(
        &control_lane,
        &branch_lane,
        &[
            Field::ResponseDigest,
            Field::Output(Output::Handoff),
            Field::Output(Output::SurfaceContract),
            Field::Output(Output::Declaration),
            Field::Output(Output::SupportPosture),
            Field::Output(Output::Workspace),
        ],
    );
    assert_equal_on(
        &detail_lane,
        &table_lane,
        &[
            Field::Output(Output::SurfaceContract),
            Field::ResponseDigest,
            Field::Output(Output::Handoff),
            Field::Output(Output::SupportPosture),
            Field::Output(Output::Branch),
            Field::Output(Output::Workspace),
            Field::Output(Output::SupportMatrix),
        ],
    );
    assert_not_equal_on(
        &detail_lane,
        &table_lane,
        &[
            Field::Output(Output::Declaration),
            Field::Output(Output::ViewShape),
        ],
    );
    assert_not_equal_on(
        &control_lane,
        &remask_lane,
        &[
            Field::ResponseDigest,
            Field::Output(Output::SupportPosture),
            Field::Output(Output::Handoff),
        ],
    );

    assert_equal_on(
        &runtime_backed_lane,
        &compatibility_runtime_backed_lane,
        &[
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
            Field::Output(Output::DeliveryRequest),
            Field::Output(Output::ResumeMode),
            Field::Output(Output::FreshnessMode),
            Field::Output(Output::DeliveryClass),
        ],
    );
    assert_equal_on(
        &durable_later_lane,
        &compatibility_durable_later_lane,
        &[
            Field::ResponseDigest,
            Field::FailureDigest,
            Field::CounterSnapshot,
            Field::Output(Output::Declaration),
            Field::Output(Output::Basis),
            Field::Output(Output::Branch),
            Field::Output(Output::Workspace),
            Field::Output(Output::DeliveryRequest),
            Field::Output(Output::ResumeMode),
            Field::Output(Output::FreshnessMode),
            Field::Output(Output::DeliveryClass),
            Field::Output(Output::DenialCode),
            Field::Output(Output::DenialDetail),
        ],
    );
    assert_equal_on(
        &runtime_backed_lane,
        &durable_later_lane,
        &[
            Field::Output(Output::Declaration),
            Field::Output(Output::Basis),
            Field::Output(Output::Branch),
            Field::Output(Output::Workspace),
            Field::Output(Output::FreshnessMode),
            Field::Output(Output::DeliveryClass),
        ],
    );
    assert_not_equal_on(
        &runtime_backed_lane,
        &durable_later_lane,
        &[
            Field::ResponseDigest,
            Field::FailureDigest,
            Field::Output(Output::DeliveryRequest),
            Field::Output(Output::ResumeMode),
        ],
    );
    assert_eq!(
        durable_later_lane.output_digest(Output::DenialCode),
        Some("DurableResumeDeferred")
    );
    assert_eq!(
        compatibility_durable_later_lane.output_digest(Output::DenialCode),
        Some("DurableResumeDeferred")
    );

    assert_counter_exact(&control_lane, "response.success.count", 1);
    assert_counter_exact(&equivalent_lane, "response.success.count", 1);
    assert_counter_exact(&compatibility_lane, "response.success.count", 1);
    assert_counter_exact(&forensic_lane, "response.success.count", 1);
    assert_counter_exact(&branch_lane, "response.success.count", 1);
    assert_counter_exact(&detail_lane, "response.success.count", 1);
    assert_counter_exact(&table_lane, "response.success.count", 1);
    assert_counter_exact(&runtime_backed_lane, "response.success.count", 1);
    assert_counter_exact(
        &compatibility_runtime_backed_lane,
        "response.success.count",
        1,
    );
    assert_counter_exact(&remask_lane, "response.success.count", 1);
    assert_counter_exact(
        &durable_later_lane,
        "response.query_handoff_denial.count",
        1,
    );
    assert_counter_exact(
        &compatibility_durable_later_lane,
        "response.query_handoff_denial.count",
        1,
    );
    assert_counter_exact(
        &retained_denial_lane,
        "response.query_handoff_denial.count",
        1,
    );
    assert_not_equal_on(
        &control_lane,
        &retained_denial_lane,
        &[Field::ResponseDigest, Field::FailureDigest],
    );
    assert!(remask_lane.output_digest(Output::Remask).is_some());
    assert_eq!(
        remask_lane.output_digest(Output::Policy),
        Some("policy:test")
    );
}
