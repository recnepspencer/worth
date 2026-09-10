use worth_query_host::facade::{declaration, domain, primary_graph};

use super::super::schema::*;

pub(super) fn seed_graph(
    graph: &mut primary_graph::WorthQueryPrimaryGraphBootstrap<TemporalHostSchema>,
    principal_binding: &domain::WorthQueryInstalledPrincipalBinding<
        TemporalHostSchema,
        TemporalPrincipalBinding,
        ExternalMapping,
        Principal,
        u64,
    >,
    gate: &str,
    unrelated_row_count: usize,
    intent_row_count: usize,
    include_live_relations: bool,
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
    for ordinal in 1..=intent_row_count {
        let row = format!("intent-row-{ordinal}");
        let identity = format!("intent-{ordinal}");
        graph
            .bind_entity(
                primary_graph::WorthQueryApplicationEntitySeed::new(
                    TemporalIntent::reference(),
                    primary_graph::WorthQueryApplicationEntityKey::new(row.clone()).unwrap(),
                )
                .field(IntentIdentityField::reference(), identity)
                .field(IntentRevisionField::reference(), 1_u64)
                .field(IntentDueField::reference(), 5_u64)
                .field(IntentLifecycleField::reference(), "active".to_string())
                .field(IntentInputField::reference(), "payload".to_string())
                .field(IntentGateField::reference(), gate.to_string())
                .field(IntentEffectField::reference(), "pending".to_string()),
            )
            .unwrap();
        if include_live_relations {
            graph
                .bind_relation(primary_graph::WorthQueryApplicationRelationSeed::new(
                    IntentLiveTarget::reference(),
                    format!("intent-live-target-{ordinal}"),
                    primary_graph::WorthQueryApplicationEntityKey::new(row.clone()).unwrap(),
                    primary_graph::WorthQueryApplicationEntityKey::new(row).unwrap(),
                ))
                .unwrap();
        }
    }
    for ordinal in 0..unrelated_row_count {
        graph
            .bind_entity(
                primary_graph::WorthQueryApplicationEntitySeed::new(
                    UnrelatedRecord::reference(),
                    primary_graph::WorthQueryApplicationEntityKey::new(format!(
                        "unrelated-{ordinal}"
                    ))
                    .unwrap(),
                )
                .field(UnrelatedValueField::reference(), ordinal as u64),
            )
            .unwrap();
    }
}
