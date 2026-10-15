use std::sync::atomic::Ordering;

use worth_foundational::facade::DiagnosticRichnessProfile;
use worth_server::WorthServerQueryHandoffDenialCode;

use {crate::shared_support_compat_http_phase_thirteen_assertions::{
        assert_bundle_digests_equal, assert_bundle_digests_not_equal, assert_denial_contains,
    }, crate::shared_support_compat_http_phase_thirteen_bundle::{
        WorthServerPhaseThirteenBundle, AUDIT_EVIDENCE_DIGEST, BASIS_DIGEST, DECLARATION_DIGEST,
        FAILURE_DIGEST, MUTATION_RESULT_DIGEST, POLICY_DIGEST, PROVENANCE_DIGEST, RESPONSE_DIGEST,
        SUPPORT_POSTURE_DIGEST,
    }, crate::shared_support_compat_http_phase_thirteen_runtime::{
        buffered_export, compat_read_with_diagnostics, direct_and_compat_mutation,
        direct_and_compat_read, finished_incremental_export, idempotent_mutation,
        phase_thirteen_counting_server, phase_thirteen_server,
    }, crate::shared_support_compat_http_phase_two_runtime as compat_http_phase_two_runtime, crate::shared_support_worth_native_assertions as worth_native_assertions};

#[test]
fn compat_http_phase_thirteen_cross_surface_parity_matrix_stays_narrow_and_retry_honest() {
    let server = phase_thirteen_server();
    let (declaration, direct_read, compat_read) = direct_and_compat_read(&server, "users.profile");
    let incremental = finished_incremental_export(&server, "users.profile", 11);
    let buffered = buffered_export(&server, "users.profile");
    let forensic_read = compat_read_with_diagnostics(
        &server,
        "users.profile",
        DiagnosticRichnessProfile::Forensic,
    );
    let basis_denial =
        match server
            .compat_http()
            .read(worth_server::WorthServerCompatibilityExecutionInput::new(
                compat_http_phase_two_runtime::prepared_read_request(
                    &server,
                    compat_http_phase_two_runtime::read_input("users.profile")
                        .with_query_pair("basis", "basis:drifted")
                        .build()
                        .expect("drifted basis request should validate"),
                ),
                "users.profile",
            )) {
            worth_proof::TransitionOutcome::Denied(value) => value,
            other => panic!("expected basis denial, got {other:?}"),
        };

    let direct_read_bundle = WorthServerPhaseThirteenBundle::new()
        .with_digest(DECLARATION_DIGEST, declaration.declaration_digest())
        .with_optional_digest(BASIS_DIGEST, direct_read.direct_context().basis_digest())
        .with_digest(
            SUPPORT_POSTURE_DIGEST,
            direct_read.direct_context().support_posture_digest(),
        )
        .with_digest(
            PROVENANCE_DIGEST,
            worth_native_assertions::direct_provenance_digest(
                direct_read.direct_context().provenance(),
            ),
        )
        .with_digest(
            AUDIT_EVIDENCE_DIGEST,
            worth_native_assertions::operator_evidence_record(
                &server,
                direct_read.response_envelope().clone(),
            )
            .response_digest(),
        );
    let compat_read_bundle = WorthServerPhaseThirteenBundle::new()
        .with_digest(DECLARATION_DIGEST, compat_read.declaration_digest())
        .with_optional_digest(BASIS_DIGEST, compat_read.direct_context().basis_digest())
        .with_digest(
            SUPPORT_POSTURE_DIGEST,
            compat_read.direct_context().support_posture_digest(),
        )
        .with_digest(
            PROVENANCE_DIGEST,
            worth_native_assertions::direct_provenance_digest(
                compat_read.direct_context().provenance(),
            ),
        )
        .with_digest(
            RESPONSE_DIGEST,
            compat_read.response_envelope().canonical_digest(),
        )
        .with_digest(
            POLICY_DIGEST,
            compat_read
                .file_envelope()
                .policy_decision()
                .canonical_digest(),
        )
        .with_digest(
            AUDIT_EVIDENCE_DIGEST,
            compat_read
                .certification_bundle()
                .operator_evidence_record()
                .canonical_digest(),
        );
    let incremental_bundle = WorthServerPhaseThirteenBundle::new()
        .with_digest(DECLARATION_DIGEST, incremental.read().declaration_digest())
        .with_optional_digest(
            BASIS_DIGEST,
            incremental.read().direct_context().basis_digest(),
        )
        .with_digest(
            SUPPORT_POSTURE_DIGEST,
            incremental.read().direct_context().support_posture_digest(),
        )
        .with_digest(
            PROVENANCE_DIGEST,
            worth_native_assertions::direct_provenance_digest(
                incremental.read().direct_context().provenance(),
            ),
        )
        .with_digest(
            RESPONSE_DIGEST,
            incremental.read().response_envelope().canonical_digest(),
        )
        .with_digest(
            POLICY_DIGEST,
            incremental
                .file_envelope()
                .policy_decision()
                .canonical_digest(),
        )
        .with_digest(
            AUDIT_EVIDENCE_DIGEST,
            incremental
                .certification_bundle()
                .operator_evidence_record()
                .canonical_digest(),
        );
    let buffered_bundle = WorthServerPhaseThirteenBundle::new()
        .with_digest(DECLARATION_DIGEST, buffered.read().declaration_digest())
        .with_optional_digest(
            BASIS_DIGEST,
            buffered.read().direct_context().basis_digest(),
        )
        .with_digest(
            SUPPORT_POSTURE_DIGEST,
            buffered.read().direct_context().support_posture_digest(),
        )
        .with_digest(
            PROVENANCE_DIGEST,
            worth_native_assertions::direct_provenance_digest(
                buffered.read().direct_context().provenance(),
            ),
        )
        .with_digest(
            RESPONSE_DIGEST,
            buffered.read().response_envelope().canonical_digest(),
        )
        .with_digest(
            POLICY_DIGEST,
            buffered
                .file_envelope()
                .policy_decision()
                .canonical_digest(),
        )
        .with_digest(
            AUDIT_EVIDENCE_DIGEST,
            buffered
                .certification_bundle()
                .operator_evidence_record()
                .canonical_digest(),
        );
    let forensic_bundle = WorthServerPhaseThirteenBundle::new()
        .with_digest(DECLARATION_DIGEST, forensic_read.declaration_digest())
        .with_optional_digest(BASIS_DIGEST, forensic_read.direct_context().basis_digest())
        .with_digest(
            SUPPORT_POSTURE_DIGEST,
            forensic_read.direct_context().support_posture_digest(),
        )
        .with_digest(
            PROVENANCE_DIGEST,
            worth_native_assertions::direct_provenance_digest(
                forensic_read.direct_context().provenance(),
            ),
        )
        .with_digest(
            POLICY_DIGEST,
            forensic_read
                .file_envelope()
                .policy_decision()
                .canonical_digest(),
        );

    assert_bundle_digests_equal(
        &direct_read_bundle,
        &compat_read_bundle,
        &[
            DECLARATION_DIGEST,
            BASIS_DIGEST,
            SUPPORT_POSTURE_DIGEST,
            PROVENANCE_DIGEST,
        ],
    );
    assert_bundle_digests_equal(
        &compat_read_bundle,
        &incremental_bundle,
        &[
            DECLARATION_DIGEST,
            BASIS_DIGEST,
            SUPPORT_POSTURE_DIGEST,
            PROVENANCE_DIGEST,
            RESPONSE_DIGEST,
        ],
    );
    assert_bundle_digests_equal(
        &compat_read_bundle,
        &buffered_bundle,
        &[
            DECLARATION_DIGEST,
            BASIS_DIGEST,
            SUPPORT_POSTURE_DIGEST,
            PROVENANCE_DIGEST,
            RESPONSE_DIGEST,
        ],
    );
    assert_bundle_digests_equal(
        &compat_read_bundle,
        &forensic_bundle,
        &[
            DECLARATION_DIGEST,
            BASIS_DIGEST,
            SUPPORT_POSTURE_DIGEST,
            PROVENANCE_DIGEST,
        ],
    );

    let (counting_server, attempted_writes) = phase_thirteen_counting_server();
    let (direct_mutation, compat_mutation) =
        direct_and_compat_mutation(&counting_server, "phase-thirteen-cross-surface");
    let first_retry_attempt =
        idempotent_mutation(&counting_server, "phase-thirteen-retry", "phase-13-idem");
    let resolved_retry =
        idempotent_mutation(&counting_server, "phase-thirteen-retry", "phase-13-idem");

    let direct_mutation_bundle = WorthServerPhaseThirteenBundle::new()
        .with_digest(
            MUTATION_RESULT_DIGEST,
            direct_mutation.mutation_result().result_digest(),
        )
        .with_digest(
            SUPPORT_POSTURE_DIGEST,
            direct_mutation.direct_context().support_posture_digest(),
        )
        .with_digest(
            PROVENANCE_DIGEST,
            worth_native_assertions::direct_provenance_digest(
                direct_mutation.direct_context().provenance(),
            ),
        );
    let compat_mutation_bundle = WorthServerPhaseThirteenBundle::new()
        .with_digest(
            MUTATION_RESULT_DIGEST,
            compat_mutation.mutation_result().result_digest(),
        )
        .with_digest(
            SUPPORT_POSTURE_DIGEST,
            compat_mutation
                .envelope()
                .direct_context()
                .support_posture_digest(),
        )
        .with_digest(
            PROVENANCE_DIGEST,
            worth_native_assertions::direct_provenance_digest(
                compat_mutation.envelope().direct_context().provenance(),
            ),
        );
    let first_retry_bundle = WorthServerPhaseThirteenBundle::new()
        .with_digest(
            MUTATION_RESULT_DIGEST,
            first_retry_attempt.mutation_result().result_digest(),
        )
        .with_digest(
            RESPONSE_DIGEST,
            first_retry_attempt
                .envelope()
                .response_envelope()
                .canonical_digest(),
        )
        .with_digest(
            SUPPORT_POSTURE_DIGEST,
            first_retry_attempt
                .envelope()
                .direct_context()
                .support_posture_digest(),
        )
        .with_digest(
            PROVENANCE_DIGEST,
            worth_native_assertions::direct_provenance_digest(
                first_retry_attempt.envelope().direct_context().provenance(),
            ),
        );
    let resolved_retry_bundle = WorthServerPhaseThirteenBundle::new()
        .with_digest(
            MUTATION_RESULT_DIGEST,
            resolved_retry.mutation_result().result_digest(),
        )
        .with_digest(
            RESPONSE_DIGEST,
            resolved_retry
                .envelope()
                .response_envelope()
                .canonical_digest(),
        )
        .with_digest(
            SUPPORT_POSTURE_DIGEST,
            resolved_retry
                .envelope()
                .direct_context()
                .support_posture_digest(),
        )
        .with_digest(
            PROVENANCE_DIGEST,
            worth_native_assertions::direct_provenance_digest(
                resolved_retry.envelope().direct_context().provenance(),
            ),
        );

    assert_bundle_digests_equal(
        &direct_mutation_bundle,
        &compat_mutation_bundle,
        &[PROVENANCE_DIGEST],
    );
    assert_bundle_digests_not_equal(
        &direct_mutation_bundle,
        &compat_mutation_bundle,
        &[SUPPORT_POSTURE_DIGEST],
    );
    assert_bundle_digests_equal(
        &first_retry_bundle,
        &resolved_retry_bundle,
        &[
            MUTATION_RESULT_DIGEST,
            RESPONSE_DIGEST,
            SUPPORT_POSTURE_DIGEST,
            PROVENANCE_DIGEST,
        ],
    );
    assert!(!first_retry_attempt
        .envelope()
        .retry_receipt()
        .is_previously_completed());
    assert!(resolved_retry
        .envelope()
        .retry_receipt()
        .is_previously_completed());
    assert_eq!(
        attempted_writes.load(Ordering::Relaxed),
        3,
        "direct write, compat write, and the first idempotent mutation should be the only authority effects",
    );

    assert_denial_contains(
        &basis_denial,
        WorthServerQueryHandoffDenialCode::CompatibilityBasisRequestInvalid,
        "drifted from the admitted retained basis",
    );
    let basis_denial_bundle = WorthServerPhaseThirteenBundle::new().with_digest(
        FAILURE_DIGEST,
        format!("{:?}:{}", basis_denial.code(), basis_denial.detail()),
    );
    assert!(basis_denial_bundle
        .digest(FAILURE_DIGEST)
        .expect("basis denial bundle should preserve failure digest")
        .contains("CompatibilityBasisRequestInvalid"),);
}
