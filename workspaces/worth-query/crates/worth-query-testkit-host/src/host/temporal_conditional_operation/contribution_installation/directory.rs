//! The ordinary temporal contribution installed through the real Home opener.
use super::*;
use worth_query_host::facade::{
    WorthQueryApplicationHomeLocation, WorthQueryApplicationHomeRequest,
};

pub fn request(
    location: std::path::PathBuf,
) -> WorthQueryApplicationHomeRequest<'static, TemporalHostSchema, TemporalInstallationProgram> {
    request_at(WorthQueryApplicationHomeLocation::Directory(location), false)
}

pub fn memory_request(
) -> WorthQueryApplicationHomeRequest<'static, TemporalHostSchema, TemporalInstallationProgram> {
    request_at(WorthQueryApplicationHomeLocation::MemoryOnly, false)
}

pub fn colliding_memory_request(
) -> WorthQueryApplicationHomeRequest<'static, TemporalHostSchema, TemporalInstallationProgram> {
    request_at(WorthQueryApplicationHomeLocation::MemoryOnly, true)
}

fn request_at(
    location: WorthQueryApplicationHomeLocation,
    colliding_intents: bool,
) -> WorthQueryApplicationHomeRequest<'static, TemporalHostSchema, TemporalInstallationProgram> {
    let mut config = publication_limit::configuration();
    config.multi_intent_targets = colliding_intents;
    WorthQueryApplicationHomeRequest::new(
        location,
        validated_program(),
        application_installation::WorthQueryApplicationProgramRoster::new(),
        TemporalHostSchema::declaration().unwrap(),
        (config,),
        ApplicationBuildIdentity::parse(env!("WORTH_IMPLEMENTATION_BUILD_ID")).unwrap(),
        WorthQueryApplicationLimits::new(
            product_world_resources(1_024).with_execution_policy(
                runtime::ExecutionRequestPolicy::new(
                    runtime::ExecutionPosture::Serial,
                    runtime::DeterminismContract::CanonicalBitwise,
                    runtime::ExecutionBudget::new(std::num::NonZeroUsize::MIN, None, None),
                ),
            ),
            runtime::WorthQueryApplicationQueryResourceProfile::default(),
            primary_graph::SignalConditionalEvaluationBudget::development(),
        ),
        move |_, graph, installed| {
            let principal = installed
                .principal_binding(TemporalPrincipalBinding::reference())
                .unwrap();
            if colliding_intents {
                seed_colliding_intents(graph, &principal);
            } else {
                seed_graph(graph, &principal, "ready", 0, 1, true);
            }
            Ok(())
        },
    )
}

fn seed_colliding_intents(
    graph: &mut primary_graph::WorthQueryPrimaryGraphBootstrap<TemporalHostSchema>,
    principal_binding: &domain::WorthQueryInstalledPrincipalBinding<
        TemporalHostSchema,
        TemporalPrincipalBinding,
        ExternalMapping,
        Principal,
        u64,
        declaration::application_schema::U64ApplicationValueBinding,
    >,
) {
    graph
        .bind_principal(
            principal_binding,
            primary_graph::WorthQueryApplicationPrincipalKey::new("temporal-host").unwrap(),
            1_u64,
            declaration::authentication::WorthQueryExternalPrincipalIdentity::new(
                "https://issuer.example",
                "temporal-host",
            )
            .unwrap(),
            declaration::authentication::WorthQueryPrincipalMappingStatus::Enabled,
        )
        .unwrap();
    let scope_row = "intent-row-1".to_string();
    for (ordinal, (due, input)) in [
        (5_u64, "shared:payload"),
        (15, "shared:other-payload"),
        (25, "shared:third-payload"),
    ]
    .into_iter()
    .enumerate()
    {
        let ordinal = ordinal + 1;
        let row = format!("intent-row-{ordinal}");
        let identity = format!("intent-{ordinal}");
        graph
            .bind_entity(
                primary_graph::WorthQueryApplicationEntitySeed::new(
                    TemporalIntent::reference(),
                    primary_graph::WorthQueryApplicationEntityKey::new(row.clone()).unwrap(),
                )
                .field(IntentIdentityField::at("IntentIdentityField"), identity)
                .field(IntentRevisionField::at("IntentRevisionField"), 1_u64)
                .field(IntentDueField::at("IntentDueField"), due)
                .field(
                    IntentLifecycleField::at("IntentLifecycleField"),
                    "active".to_string(),
                )
                .field(IntentInputField::at("IntentInputField"), input.to_string())
                .field(IntentGateField::at("IntentGateField"), "ready".to_string())
                .field(
                    IntentEffectField::at("IntentEffectField"),
                    "pending".to_string(),
                ),
            )
            .unwrap();
        // Link the scoped intent ("intent-1") to all three targets so
        // `TemporalIntentTargetsQuery` projects all three distinct intents on
        // the same Memory branch and conditional definition.
        graph
            .bind_relation(primary_graph::WorthQueryApplicationRelationSeed::new(
                IntentLiveTarget::reference(),
                format!("intent-live-target-{ordinal}"),
                primary_graph::WorthQueryApplicationEntityKey::new(scope_row.clone()).unwrap(),
                primary_graph::WorthQueryApplicationEntityKey::new(row).unwrap(),
            ))
            .unwrap();
    }
}
