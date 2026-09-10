use std::sync::Arc;

use super::super::support::*;
use crate::domain_capabilities::{
    admit_eligible_domain_capability_contribution,
    evaluate_requested_domain_capability_contribution,
    materialize_query_invariant_catalog_registration_artifact,
    prepare_admitted_domain_capability_contribution_for_materialization,
    WorthQueryInvariantCapabilityContributionAuthoring,
};
use worth_proof::TransitionOutcome;
use worth_relational::facade::runtime::RelationalRuntimeApi;

struct CertificationBoundaryViolationRule;

impl CustomInvariantRule for CertificationBoundaryViolationRule {
    type Scope = ();

    fn descriptor(&self) -> CustomInvariantDescriptor {
        CustomInvariantDescriptor {
            identity: CustomInvariantSemanticIdentity {
                rule_id: CustomInvariantRuleId::new("query.test.certification-violation"),
                semantic_version: CustomInvariantSemanticVersion::new(1, 0),
            },
            display_name: Arc::from("Query Test Certification Violation"),
            operational: CustomInvariantOperationalMetadata {
                execution_point: InvariantExecutionPoint::CertificationBoundary,
                groups: InvariantGroupSet::of(InvariantGroup::PublicationCoherence),
                cost_class: InvariantCostClass::Touched,
                failure_effect: InvariantFailureEffect::BlockPublication,
            },
        }
    }

    fn prepare_scope(
        &self,
        _planner: &mut CustomInvariantScopePlanner<'_>,
    ) -> Result<Self::Scope, CustomInvariantPreparationError> {
        Ok(())
    }

    fn evaluate(
        &self,
        _context: &CustomInvariantExecutionContext<'_>,
        _scope: &Self::Scope,
    ) -> Result<CustomInvariantVerdict, CustomInvariantExecutionError> {
        Ok(CustomInvariantVerdict::Violation)
    }
}

struct InspectingInvariantWriteAuthority;

impl WorthQueryRuntimeWriteAuthorityAdapter for InspectingInvariantWriteAuthority {
    fn write(
        &mut self,
        bridge: &RuntimeBridge,
        relational_runtime: Option<&mut RelationalRuntime>,
        mutation: WorthQueryBackendAdmissibleMutation,
    ) -> Result<WriteAuthorityExecutionReceipt, WorthQueryWorkspaceError> {
        let relational_runtime = relational_runtime
            .expect("query-owned invariant registration should lower a relational runtime");
        let certification = relational_runtime.validation().certification_state();
        assert_eq!(certification.summary().result_count(), 1);
        assert_eq!(certification.summary().violation_count(), 1);
        assert!(certification.summary().publication_failure().is_some());

        let result = certification
            .results()
            .first()
            .expect("custom certification invariant result");
        assert_eq!(
            result.execution_point,
            InvariantExecutionPoint::CertificationBoundary
        );
        assert_eq!(
            result.failure_effect,
            InvariantFailureEffect::BlockPublication
        );
        assert!(format!("{:?}", result.rule).contains("query.test.certification-violation"));
        assert!(format!("{:?}", result.verdict).contains("Violation"));

        let mut authority = TestWriteAuthority;
        authority.write(bridge, Some(relational_runtime), mutation)
    }
}

struct InspectingCatalogWriteAuthority {
    expected_catalog: InvariantCatalog,
}

impl WorthQueryRuntimeWriteAuthorityAdapter for InspectingCatalogWriteAuthority {
    fn write(
        &mut self,
        bridge: &RuntimeBridge,
        relational_runtime: Option<&mut RelationalRuntime>,
        mutation: WorthQueryBackendAdmissibleMutation,
    ) -> Result<WriteAuthorityExecutionReceipt, WorthQueryWorkspaceError> {
        let relational_runtime = relational_runtime
            .expect("query-owned invariant catalog should lower a relational runtime");
        assert_eq!(
            relational_runtime.config().schema.invariant_catalog,
            self.expected_catalog
        );

        let mut authority = TestWriteAuthority;
        authority.write(bridge, Some(relational_runtime), mutation)
    }
}

#[test]
fn query_builder_register_invariant_lowers_custom_rule_into_relational_runtime() {
    let mut runtime = complete_query_owned_backend_from_parts_builder()
        .register_invariant(CertificationBoundaryViolationRule)
        .expect("custom query invariant registration should succeed")
        .write_authority(InspectingInvariantWriteAuthority)
        .build_backend_from_parts()
        .build()
        .expect("query runtime with registered invariant should build");

    runtime
        .write(insert_command(
            "Task",
            [
                ("identity.id", test_string_aspect_value("query-invariant-1")),
                (
                    "title.value",
                    test_string_aspect_value("Invariant test task"),
                ),
            ],
        ))
        .expect("write should exercise invariant-aware relational runtime");
}

#[test]
fn query_builder_invariant_catalog_lowers_into_relational_runtime_config() {
    let expected_catalog = InvariantCatalog {
        registrations: vec![InvariantRegistration::commit_boundary_blocking(
            InvariantRule::MaxMergedIntents(7),
        )],
    };

    let mut runtime = complete_query_owned_backend_from_parts_builder()
        .invariant_catalog(expected_catalog.clone())
        .write_authority(InspectingCatalogWriteAuthority {
            expected_catalog: expected_catalog.clone(),
        })
        .build_backend_from_parts()
        .build()
        .expect("query runtime with invariant catalog should build");

    runtime
        .write(insert_command(
            "Task",
            [
                ("identity.id", test_string_aspect_value("query-catalog-1")),
                ("title.value", test_string_aspect_value("Catalog test task")),
            ],
        ))
        .expect("write should expose lowered invariant catalog");
}

#[test]
fn query_builder_rejects_explicit_relational_runtime_when_query_owned_invariants_are_queued() {
    let error = match WorthQueryRuntime::builder(test_product_world_resources())
        .register_invariant(CertificationBoundaryViolationRule)
        .expect("custom query invariant registration should succeed")
        .relational_runtime(RelationalRuntimeApi::builder().build())
        .build_backend_from_parts()
        .build()
    {
        Ok(_) => {
            panic!("explicit relational runtime should conflict with queued query-owned invariants")
        }
        Err(error) => error,
    };

    match error {
        WorthQueryRuntimeError::InvariantRegistration { stage, message } => {
            assert_eq!(stage, "relational_runtime_authority_selection");
            assert!(message.contains("explicitly supplied relational runtime"));
            assert!(message.contains("choose one authority path"));
        }
        other => panic!("unexpected runtime error: {other:?}"),
    }
}

#[test]
fn query_builder_rejects_explicit_backend_when_query_owned_invariants_are_queued() {
    let explicit_backend = WorthQueryBridgeBackedRuntimeBackend::from_parts(
        WorthQueryRuntimeBackendParts::new()
            .runtime_bridge(test_bridge())
            .schema_adapter(TestSchemaAdapter)
            .source_adapter(TestSourceAdapter::default())
            .write_authority(TestWriteAuthority)
            .snapshot_identity(TestSnapshotIdentityAdapter)
            .signal_sink(TestSignalSink)
            .subscription_activation(TestSubscriptionActivation)
            .preview_basis(TestPreviewBasis)
            .inspector_evidence(TestInspectorEvidence),
    )
    .expect("explicit backend should build for invariant-lane conflict test");

    let error = match WorthQueryRuntime::builder(test_product_world_resources())
        .register_invariant(CertificationBoundaryViolationRule)
        .expect("custom query invariant registration should succeed")
        .backend(explicit_backend)
        .build()
    {
        Ok(_) => panic!("explicit backend should reject queued query-owned invariants"),
        Err(error) => error,
    };

    match error {
        WorthQueryRuntimeError::InvariantRegistration { stage, message } => {
            assert_eq!(stage, "runtime_backend_selection");
            assert!(message.contains("queued Query-owned invariant registrations"));
            assert!(message.contains("backend(...)"));
            assert!(message.contains("relational_runtime(...)"));
        }
        other => panic!("unexpected runtime error: {other:?}"),
    }
}

#[test]
fn query_builder_accepts_proof_lane_invariant_registration_artifact() {
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
        .build()
        .expect("query runtime should accept proof-lane invariant registration artifacts");

    runtime
        .write(insert_command(
            "Task",
            [
                ("identity.id", test_string_aspect_value("query-artifact-1")),
                (
                    "title.value",
                    test_string_aspect_value("Artifact-backed task"),
                ),
            ],
        ))
        .expect("artifact-backed invariant registration should lower into runtime config");
}

#[test]
fn query_builder_canonicalizes_and_deduplicates_merged_invariant_catalog_sources() {
    let declaration = sample_declaration("runtime-canonical-merge");
    let requested = WorthQueryInvariantCapabilityContributionAuthoring::invariant_registration(
        InvariantCatalog {
            registrations: vec![
                InvariantRegistration::commit_boundary_blocking(InvariantRule::MaxMergedIntents(9)),
                InvariantRegistration::commit_boundary_blocking(InvariantRule::MaxMergedIntents(3)),
            ],
        },
        "runtime.artifact.invariant-registration.merge",
        "merged invariant sources should preserve one canonical catalog identity",
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

    let expected_catalog = InvariantCatalog {
        registrations: vec![
            InvariantRegistration::commit_boundary_blocking(InvariantRule::MaxMergedIntents(3)),
            InvariantRegistration::commit_boundary_blocking(InvariantRule::MaxMergedIntents(9)),
        ],
    };

    let mut runtime = complete_query_owned_backend_from_parts_builder()
        .invariant_catalog(InvariantCatalog {
            registrations: vec![InvariantRegistration::commit_boundary_blocking(
                InvariantRule::MaxMergedIntents(9),
            )],
        })
        .invariant_registration_artifact(artifact)
        .write_authority(InspectingCatalogWriteAuthority {
            expected_catalog: expected_catalog.clone(),
        })
        .build_backend_from_parts()
        .build()
        .expect("merged invariant catalogs should lower canonically");

    runtime
        .write(insert_command(
            "Task",
            [
                (
                    "identity.id",
                    test_string_aspect_value("query-canonical-merge-1"),
                ),
                (
                    "title.value",
                    test_string_aspect_value("Canonical merge task"),
                ),
            ],
        ))
        .expect("merged invariant catalogs should stay executable");
}

fn sample_declaration(name: &str) -> WorthQueryIntentDeclaration {
    WorthQueryIntentDeclaration::strategy_commit(
        name,
        format!("worth.spatial.{name}"),
        "1",
        "worth.spatial.intent",
        WorthQueryIntentInput::object([(
            "entity",
            WorthQueryIntentInput::string(format!("edge:{name}")),
        )]),
    )
}

fn success<T, D, S, R, F, O>(outcome: TransitionOutcome<T, D, S, R, F, O>) -> T
where
    D: std::fmt::Debug,
    S: std::fmt::Debug,
    R: std::fmt::Debug,
    F: std::fmt::Debug,
{
    match outcome {
        TransitionOutcome::Success(value) => value,
        TransitionOutcome::Denied(denial) => panic!("expected success, got denial: {denial:?}"),
        TransitionOutcome::Stale(stale) => panic!("expected success, got stale: {stale:?}"),
        TransitionOutcome::RebindRequired(rebind) => {
            panic!("expected success, got rebind-required: {rebind:?}")
        }
        _ => panic!("expected success, got non-success transition outcome"),
    }
}
