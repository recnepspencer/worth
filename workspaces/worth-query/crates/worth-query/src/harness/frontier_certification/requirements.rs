use super::{FrontierCloseoutRequirement, FrontierCloseoutStatus};

pub(super) fn must_ship_requirements() -> Vec<FrontierCloseoutRequirement> {
    vec![
        FrontierCloseoutRequirement {
            requirement_name: "typed frontier planning and serial fallback families",
            status: FrontierCloseoutStatus::Satisfied,
            proof_artifacts: &[
                "FrontierAwarePlan (test-only)",
                "SerialFallbackRoute (test-only)",
                "FrontierParityBundle (test-only)",
            ],
            certification_rows: &[
                "frontier-serial-control",
                "serial-fallback-parity",
            ],
            notes: "Rows frontier-serial-control and serial-fallback-parity assert the ordered plan family and exact fallback report reason/drift; frontier_certification_lanes_preserve_exact_identities_and_results checks route families and baseline digests.",
        },
        FrontierCloseoutRequirement {
            requirement_name: "frontier-aware lowering and packet identity",
            status: FrontierCloseoutStatus::Satisfied,
            proof_artifacts: &[
                "FrontierAwarePlan (test-only)",
                "PlannedWorkPacket (test-only)",
                "PacketMergeBoundary (test-only)",
                "FrontierPlanningReport (test-only)",
            ],
            certification_rows: &[
                "frontier-serial-control",
            ],
            notes: "The frontier-serial-control builder checks ordered root packet and merge contract, repeat packet/posture identity, different-order packet identity, and live descriptor identity. Ordered lowering unit tests also distinguish source plan posture digests.",
        },
        FrontierCloseoutRequirement {
            requirement_name: "typed fallback report metadata",
            status: FrontierCloseoutStatus::Satisfied,
            proof_artifacts: &[
                "FrontierPredictionDriftOutcome (test-only)",
                "SerialFallbackReason (test-only)",
                "FrontierRouteReport (test-only)",
            ],
            certification_rows: &[
                "serial-fallback-parity",
            ],
            notes: "The serial-fallback-parity builder compares report drift and reason exactly to SerialFallbackRequired and PredictionDriftRequiresSerialRoute; the reason has one evidence source.",
        },
    ]
}

pub(super) fn must_preserve_requirements() -> Vec<FrontierCloseoutRequirement> {
    vec![
        FrontierCloseoutRequirement {
            requirement_name: "canonical query and basis authority preserved",
            status: FrontierCloseoutStatus::Satisfied,
            proof_artifacts: &[
                "FrontierAwarePlan (test-only)",
                "FrontierParityBundle (test-only)",
            ],
            certification_rows: &[
                "frontier-serial-control",
                "exact-basis-bundle-parity",
            ],
            notes: "frontier_certification_lanes_preserve_exact_identities_and_results compares query, plan, basis and result to authoritative preflight/baseline values; the bundle row builder checks every member basis.",
        },
        FrontierCloseoutRequirement {
            requirement_name: "collection and live semantics remain authoritative",
            status: FrontierCloseoutStatus::Satisfied,
            proof_artifacts: &[
                "ExecutionPreflightBundle",
                "LiveQueryPlan",
                "FrontierPlanFamily (test-only)",
            ],
            certification_rows: &["frontier-serial-control"],
            notes: "The frontier-serial-control builder asserts ordered and live packet families and exact live descriptor query/plan/basis identities, also covered by live_plan_lowering_preserves_descriptor_identity_and_uses_planner_packets.",
        },
        FrontierCloseoutRequirement {
            requirement_name: "serial fallback execution preserves baseline results",
            status: FrontierCloseoutStatus::Satisfied,
            proof_artifacts: &[
                "execute_serial_fallback_route (test-only)",
                "ExecutionResultEnvelope",
            ],
            certification_rows: &[
                "serial-fallback-parity",
            ],
            notes: "frontier_certification_lanes_preserve_exact_identities_and_results compares the fallback result digest to baseline; bounded_materialization_lowers_into_serial_fallback_route_with_typed_executor_entrypoint also compares rows. This does not claim posture consumption.",
        },
    ]
}

pub(super) fn proof_obligation_requirements() -> Vec<FrontierCloseoutRequirement> {
    vec![
        FrontierCloseoutRequirement {
            requirement_name: "planned frontier shape and reported execution records are explicit",
            status: FrontierCloseoutStatus::Satisfied,
            proof_artifacts: &[
                "FrontierCounterSnapshot (test-only)",
                "FrontierParityBundle (test-only)",
            ],
            certification_rows: &["frontier-serial-control", "serial-fallback-parity", "exact-basis-bundle-parity"],
            notes: "Canonical tests assert fixture-declared breadth 3/5/10, one/two planned merge boundaries, two bundle members, fallback plan posture, and the executor-reported examined-record value. No denial, execution, reduction or preserved-work counts are claimed.",
        },
        FrontierCloseoutRequirement {
            requirement_name: "serial bundle lowering preserves exact basis",
            status: FrontierCloseoutStatus::Satisfied,
            proof_artifacts: &[
                "SerialFallbackBundleRoutes (test-only)",
                "BundleResolvedBasisDigest (test-only)",
            ],
            certification_rows: &[
                "exact-basis-bundle-parity",
                "mixed-basis-bundle-denied",
            ],
            notes: "The exact-basis-bundle-parity builder checks each member basis against the shared bundle basis; mixed_basis_bundle_rejection asserts exact expected/found digests from the real failed attempt.",
        },
        FrontierCloseoutRequirement {
            requirement_name: "drift outcomes are explicit",
            status: FrontierCloseoutStatus::Satisfied,
            proof_artifacts: &[
                "FrontierPredictionDriftOutcome (test-only)",
                "FrontierRouteReport (test-only)",
            ],
            certification_rows: &[
                "serial-fallback-parity",
            ],
            notes: "The serial-fallback-parity builder asserts exact SerialFallbackRequired report drift and fallback reason. Drift denial enforcement remains a separate unit proof in denied_by_drift_blocks_serial_route_lowering, outside this matrix claim.",
        },
    ]
}

pub(super) fn acceptance_evidence_requirements() -> Vec<FrontierCloseoutRequirement> {
    vec![
        FrontierCloseoutRequirement {
            requirement_name: "frontier planning and serial fallback parity suite passes",
            status: FrontierCloseoutStatus::Satisfied,
            proof_artifacts: &[
                "FrontierParityBundle (test-only)",
                "FrontierCounterSnapshot (test-only)",
            ],
            certification_rows: &[
                "frontier-serial-control",
                "serial-fallback-parity",
                "exact-basis-bundle-parity",
                "unsupported-frontier-family",
                "unsupported-bundle-composition",
                "mixed-basis-bundle-denied",
            ],
            notes: "frontier_certification_matrix_meets_milestone_requirements requires complete rows and nonempty outputs; exact identity/result and typed error assertions provide behavioral evidence. Closeout metadata checks only nonempty fields and known row references.",
        },
        FrontierCloseoutRequirement {
            requirement_name: "verification outputs include query, plan, result, and planned shape",
            status: FrontierCloseoutStatus::Satisfied,
            proof_artifacts: &[
                "FrontierParityBundle (test-only)",
            ],
            certification_rows: &[
                "frontier-serial-control",
                "serial-fallback-parity",
                "exact-basis-bundle-parity",
            ],
            notes: "Canonical lane tests check authoritative query/plan/result digests, literal planned breadth/boundary/member values and executor-reported records examined; rejection rows carry typed errors without synthetic snapshots.",
        },
        FrontierCloseoutRequirement {
            requirement_name: "unsupported families and mixed-basis bundles fail typed",
            status: FrontierCloseoutStatus::Satisfied,
            proof_artifacts: &[
                "FrontierPlanningError (test-only)",
            ],
            certification_rows: &[
                "unsupported-frontier-family",
                "unsupported-bundle-composition",
                "mixed-basis-bundle-denied",
            ],
            notes: "Rejection builders require exact UnsupportedFrontierFamily and UnsupportedBundleComposition variants and mixed-basis identities; frontier_certification_rejections_are_typed checks their exported failure classes. No execution ordering or denial counts are claimed.",
        },
    ]
}
