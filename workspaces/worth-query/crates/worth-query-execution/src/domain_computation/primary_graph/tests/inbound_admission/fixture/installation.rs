use super::*;

pub(in crate::domain_computation::primary_graph::tests::inbound_admission) fn installed_world_with_verifier(
    verifier: Arc<
        dyn crate::domain_computation::application_aftermath::WorthQueryInboundOccurrenceVerifier,
    >,
) -> InboundWorld {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let phase = &active_phase;

        let declaration = InboundTestSchema::declaration().unwrap();
        let package = WorthQueryPortableDomainPackage::new(WorthQueryPortableDomainIdentity::new(
            "inbound_admission_test",
            1,
            0,
        ))
        .application_schema(declaration.clone())
        .validate()
        .unwrap();
        let admitted = WorthQueryInstallationAdmissionProfile::new("support", "configuration")
            .admit(package)
            .unwrap();
        let installation = WorthQueryExecutionRuntimeInstaller::new()
            .install(WorthQueryInstallationGeneration::initial(), [admitted])
            .unwrap();
        let (runtime, authority) = installation.into_parts();
        let schema = runtime
            .installed_packages()
            .bind_application_schema(declaration)
            .unwrap();
        let binding = schema
            .principal_binding(IdentityBinding::reference())
            .unwrap();
        let mut bootstrap = authority
            .prepare_primary_graph(
                &phase.bootstrap_for_test(),
                &runtime,
                &schema,
                test_product_world_resources(),
            )
            .unwrap();
        bootstrap
            .bind_principal(
                &binding,
                WorthQueryApplicationPrincipalKey::new("principal").unwrap(),
                1_u64,
                external_identity(),
                WorthQueryPrincipalMappingStatus::Enabled,
            )
            .unwrap();
        bootstrap
            .bind_entity(
                WorthQueryApplicationEntitySeed::new(
                    Target::reference(),
                    WorthQueryApplicationEntityKey::new("target").unwrap(),
                )
                .field(TargetKey::reference(), "target".to_owned()),
            )
            .unwrap();
        let invariant = bootstrap.retain_invariant_projection_authority();
        let application = bootstrap
            .publish_application_runtime(
                &phase.bootstrap_for_test(),
                runtime,
                authority,
                schema,
                worth_signal::facade::runtime::SignalConditionalEvaluationBudget::development(),
            )
            .unwrap();
        let operation = application
            .installed_schema()
            .installed_operation(NotifyOperation::reference())
            .unwrap();
        let verifier = application
            .install_inbound_occurrence_verifier(&operation, verifier)
            .unwrap();
        InboundWorld {
            application,
            invariant,
            verifier,
        }
    })
}
