pub(crate) use {crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationField as Field, crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest as Output};
pub(crate) use {crate::certification_suite::ordinary_context::certification_counter_assertions::assert_counter_exact};
pub(crate) use {crate::certification_suite::ordinary_context::certification_digest_assertions::assert_equal_on, crate::certification_suite::ordinary_context::certification_digest_assertions::assert_not_equal_on};
pub(crate) use {crate::certification_suite::ordinary_context::certification_worth_native_fixture::worth_native_read_lanes::branch_product_read_bundle, crate::certification_suite::ordinary_context::certification_worth_native_fixture::worth_native_delivery_lanes::compatibility_durable_delivery_denial_bundle, crate::certification_suite::ordinary_context::certification_worth_native_fixture::worth_native_read_lanes::compatibility_overlap_bundle, crate::certification_suite::ordinary_context::certification_worth_native_fixture::worth_native_delivery_lanes::compatibility_runtime_backed_delivery_bundle, crate::certification_suite::ordinary_context::certification_worth_native_fixture::worth_native_delivery_lanes::durable_delivery_denial_bundle, crate::certification_suite::ordinary_context::certification_worth_native_fixture::worth_native_common::durable_later_server, crate::certification_suite::ordinary_context::certification_worth_native_fixture::worth_native_common::forensic_server, crate::certification_suite::ordinary_context::certification_worth_native_fixture::worth_native_read_lanes::lower_direct_read_bundle, crate::certification_suite::ordinary_context::certification_worth_native_fixture::worth_native_state_projection_lanes::product_projection_bundle, crate::certification_suite::ordinary_context::certification_worth_native_fixture::worth_native_read_lanes::product_read_bundle, crate::certification_suite::ordinary_context::certification_worth_native_fixture::worth_native_common::remask_server, crate::certification_suite::ordinary_context::certification_worth_native_fixture::worth_native_read_lanes::retained_artifact_denial_bundle, crate::certification_suite::ordinary_context::certification_worth_native_fixture::worth_native_delivery_lanes::runtime_backed_delivery_bundle, crate::certification_suite::ordinary_context::certification_worth_native_fixture::worth_native_common::runtime_backed_server, crate::certification_suite::ordinary_context::certification_worth_native_fixture::worth_native_common::standard_server, crate::certification_suite::ordinary_context::certification_worth_native_fixture::worth_native_read_lanes::view_shape_product_read_bundle};
pub(crate) use {worth_server::WorthServerDirectViewShape};

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
        worth_server::WorthServerDirectViewShape::Detail,
    );
    let table_lane = view_shape_product_read_bundle(
        &standard_server,
        "users.profile",
        worth_server::WorthServerDirectViewShape::Table,
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
            crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationField::Output(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::SupportMatrix),
            crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationField::Output(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::ViewShape),
        ],
    );
    assert_equal_on(
        &control_lane,
        &compatibility_lane,
        &[
            crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationField::Output(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::SurfaceContract),
            crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationField::Output(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::Declaration),
            crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationField::Output(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::DeclarationSupport),
            crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationField::Output(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::Branch),
            crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationField::Output(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::Workspace),
        ],
    );
    assert_equal_on(
        &control_lane,
        &forensic_lane,
        &[
            crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationField::Output(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::SurfaceContract),
            crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationField::Output(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::Declaration),
            crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationField::Output(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::Branch),
            crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationField::Output(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::Workspace),
            crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationField::Output(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::SupportMatrix),
            crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationField::Output(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::ViewShape),
        ],
    );
    assert_not_equal_on(
        &control_lane,
        &forensic_lane,
        &[crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationField::RequestContextDigest],
    );
    assert_not_equal_on(
        &control_lane,
        &compatibility_lane,
        &[crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationField::Output(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::SupportPosture)],
    );
    assert_not_equal_on(
        &control_lane,
        &branch_lane,
        &[crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationField::RequestContextDigest, crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationField::Output(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::Branch)],
    );
    assert_equal_on(
        &control_lane,
        &branch_lane,
        &[
            crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationField::ResponseDigest,
            crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationField::Output(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::Handoff),
            crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationField::Output(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::SurfaceContract),
            crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationField::Output(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::Declaration),
            crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationField::Output(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::SupportPosture),
            crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationField::Output(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::Workspace),
        ],
    );
    assert_equal_on(
        &detail_lane,
        &table_lane,
        &[
            crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationField::Output(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::SurfaceContract),
            crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationField::ResponseDigest,
            crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationField::Output(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::Handoff),
            crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationField::Output(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::SupportPosture),
            crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationField::Output(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::Branch),
            crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationField::Output(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::Workspace),
            crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationField::Output(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::SupportMatrix),
        ],
    );
    assert_not_equal_on(
        &detail_lane,
        &table_lane,
        &[
            crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationField::Output(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::Declaration),
            crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationField::Output(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::ViewShape),
        ],
    );
    assert_not_equal_on(
        &control_lane,
        &remask_lane,
        &[
            crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationField::ResponseDigest,
            crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationField::Output(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::SupportPosture),
            crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationField::Output(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::Handoff),
        ],
    );

    assert_equal_on(
        &runtime_backed_lane,
        &compatibility_runtime_backed_lane,
        &[
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
            crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationField::Output(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::DeliveryRequest),
            crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationField::Output(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::ResumeMode),
            crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationField::Output(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::FreshnessMode),
            crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationField::Output(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::DeliveryClass),
        ],
    );
    assert_equal_on(
        &durable_later_lane,
        &compatibility_durable_later_lane,
        &[
            crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationField::ResponseDigest,
            crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationField::FailureDigest,
            crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationField::CounterSnapshot,
            crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationField::Output(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::Declaration),
            crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationField::Output(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::Basis),
            crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationField::Output(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::Branch),
            crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationField::Output(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::Workspace),
            crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationField::Output(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::DeliveryRequest),
            crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationField::Output(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::ResumeMode),
            crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationField::Output(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::FreshnessMode),
            crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationField::Output(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::DeliveryClass),
            crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationField::Output(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::DenialCode),
            crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationField::Output(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::DenialDetail),
        ],
    );
    assert_equal_on(
        &runtime_backed_lane,
        &durable_later_lane,
        &[
            crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationField::Output(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::Declaration),
            crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationField::Output(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::Basis),
            crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationField::Output(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::Branch),
            crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationField::Output(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::Workspace),
            crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationField::Output(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::FreshnessMode),
            crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationField::Output(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::DeliveryClass),
        ],
    );
    assert_not_equal_on(
        &runtime_backed_lane,
        &durable_later_lane,
        &[
            crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationField::ResponseDigest,
            crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationField::FailureDigest,
            crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationField::Output(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::DeliveryRequest),
            crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationField::Output(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::ResumeMode),
        ],
    );
    assert_eq!(
        durable_later_lane.output_digest(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::DenialCode),
        Some("DurableResumeDeferred")
    );
    assert_eq!(
        compatibility_durable_later_lane.output_digest(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::DenialCode),
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
        &[crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationField::ResponseDigest, crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationField::FailureDigest],
    );
    assert!(remask_lane.output_digest(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::Remask).is_some());
    assert_eq!(
        remask_lane.output_digest(crate::certification_suite::ordinary_context::certification_bundle::WorthServerCertificationOutputDigest::Policy),
        Some("policy:test")
    );
}
