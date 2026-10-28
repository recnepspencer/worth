use super::*;

#[test]
fn query_builder_accepts_proof_lane_invariant_registration_artifact() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let declaration = sample_declaration("runtime-artifact");
    let requested =
        WorthQueryInvariantCapabilityContributionAuthoring::invariant_rule_registration(
            InvariantRegistration::commit_boundary_blocking(InvariantRule::MaxMergedIntents(5)),
            "runtime.artifact.invariant-registration",
            "proof-lane registration should lower through the ordinary builder",
        )
        .for_intent_declaration(&declaration);
    let eligible = success(evaluate_requested_domain_capability_contribution(requested));
    let admitted = success(admit_eligible_domain_capability_contribution(eligible));
    let ready = success(prepare_admitted_domain_capability_contribution_for_materialization(
        admitted,
        crate::domain_capabilities::WorthQueryDeclarationBoundContributionTarget::for_intent_declaration(&declaration),
    ));
    let artifact = success(materialize_query_invariant_catalog_registration_artifact(
        ready,
    ));

    let mut runtime = complete_query_owned_backend_from_parts_builder()
        .invariant_registration_artifact(artifact)
        .write_authority(InspectingCatalogWriteAuthority {
            expected_catalog: InvariantCatalog {
                registrations: vec![InvariantRegistration::commit_boundary_blocking(
                    InvariantRule::MaxMergedIntents(5),
                )],
            },
        })
        .build_backend_from_parts()
        .build(request_execution)
        .expect("query runtime should accept proof-lane invariant registration artifacts");

    runtime
        .write(
            request_execution,
            insert_command(
                "Task",
                [
                    ("identity.id", test_string_aspect_value("query-artifact-1")),
                    (
                        "title.value",
                        test_string_aspect_value("Artifact-backed task"),
                    ),
                ],
            ),
        )
        .expect("artifact-backed invariant registration should lower into runtime config");
}
