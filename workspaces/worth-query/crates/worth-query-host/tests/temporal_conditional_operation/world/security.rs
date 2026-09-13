#![allow(dead_code)] // This fixture is compiled by several independent certification targets.

use worth_query_host::facade::{declaration, primary_graph};

use super::super::{adapters::block_on, schema::*};
use super::{admit_identity_adapter, request_scope, CourtroomWorld};

impl CourtroomWorld {
    pub fn revoke_principal_on_default_product(&self) {
        let schema = self.application.installed_schema();
        let binding = schema
            .principal_binding(TemporalPrincipalBinding::reference())
            .unwrap();
        let authentication = admit_identity_adapter(schema);
        let request = request_scope();
        let external = block_on(authentication.authenticate((), &request)).unwrap();
        let principal = self
            .application
            .on_branch(self.application.current_world())
            .select()
            .expect("the selected product branch remains admitted")
            .resolve_authenticated_principal(
                &binding,
                &external,
                &request,
                primary_graph::WorthQueryPrincipalResolutionMode::Ordinary,
            )
            .unwrap();
        let mapping = self
            .application
            .on_branch(self.application.current_world())
            .select()
            .expect("the selected product branch remains admitted")
            .resolve_entity(
                ExternalIdentityField::reference(),
                declaration::authentication::WorthQueryExternalPrincipalIdentity::new(
                    "https://issuer.example",
                    "temporal-host",
                )
                .unwrap(),
                &request,
                primary_graph::WorthQueryPrincipalResolutionMode::Ordinary,
            )
            .unwrap();
        let operation = schema
            .installed_operation(RevokeTemporalPrincipal::reference())
            .unwrap();
        let admission = self
            .application
            .on_branch(self.application.current_world())
            .select()
            .unwrap()
            .authorize_operation(
                &principal,
                &mapping,
                &operation,
                Default::default(),
                &request,
            )
            .unwrap();
        let (target, projection, _) = self
            .invariant
            .project_admitted_operation(&admission, |reader, mapping| {
                reader
                    .decision_field(mapping, MappingStatusField::reference())
                    .expect("revocation admits the mapping status decision field")
                    .expect("an authenticated mapping retains its status");
                reader
                    .mutation_target(mapping)
                    .expect("the admitted mapping is a local mutation target")
            })
            .unwrap()
            .into_parts();
        let reads = self
            .application
            .begin_projected_application_read_attempt(admission, projection)
            .unwrap();
        let mut effects = reads
            .complete_projected_dependencies()
            .unwrap()
            .begin_effect_program();
        let mapping = effects.projected_entity(&target).unwrap();
        effects
            .write_field(
                &mapping,
                MappingStatusField::reference(),
                declaration::authentication::WorthQueryPrincipalMappingStatus::Disabled,
            )
            .unwrap();
        let outcome = self.application.compare_and_commit_application(
            effects.finish().unwrap(),
            primary_graph::WorthQueryApplicationIdempotencyBinding::new([0x77; 32], [0x78; 32]),
        );
        assert!(
            matches!(
                outcome,
                primary_graph::WorthQueryApplicationCommitOutcome::Committed(_)
            ),
            "revocation must publish through World: {outcome:?}"
        );
    }
}

pub(crate) fn projection_target_is_bound_to_exact_admission() {
    macro_rules! admit_revocation {
        ($world:expr) => {{
            let schema = $world.application.installed_schema();
            let binding = schema
                .principal_binding(TemporalPrincipalBinding::reference())
                .unwrap();
            let request = request_scope();
            let external =
                block_on(admit_identity_adapter(schema).authenticate((), &request)).unwrap();
            let principal = $world
                .application
                .on_branch($world.application.current_world())
                .select()
                .unwrap()
                .resolve_authenticated_principal(
                    &binding,
                    &external,
                    &request,
                    primary_graph::WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap();
            let mapping = $world
                .application
                .on_branch($world.application.current_world())
                .select()
                .unwrap()
                .resolve_entity(
                    ExternalIdentityField::reference(),
                    declaration::authentication::WorthQueryExternalPrincipalIdentity::new(
                        "https://issuer.example",
                        "temporal-host",
                    )
                    .unwrap(),
                    &request,
                    primary_graph::WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap();
            let operation = schema
                .installed_operation(RevokeTemporalPrincipal::reference())
                .unwrap();
            $world
                .application
                .on_branch($world.application.current_world())
                .select()
                .unwrap()
                .authorize_operation(
                    &principal,
                    &mapping,
                    &operation,
                    Default::default(),
                    &request,
                )
                .unwrap()
        }};
    }

    let world = CourtroomWorld::publish("ready");
    let prior_admission = admit_revocation!(world);
    let (prior_target, prior_projection, _) = world
        .invariant
        .project_admitted_operation(&prior_admission, |reader, mapping| {
            reader
                .decision_field(mapping, MappingStatusField::reference())
                .unwrap();
            reader.mutation_target(mapping).unwrap()
        })
        .unwrap()
        .into_parts();
    drop(prior_projection);

    let current_admission = admit_revocation!(world);
    let (current_target, current_projection, _) = world
        .invariant
        .project_admitted_operation(&current_admission, |reader, mapping| {
            reader
                .decision_field(mapping, MappingStatusField::reference())
                .unwrap();
            reader.mutation_target(mapping).unwrap()
        })
        .unwrap()
        .into_parts();
    let reads = world
        .application
        .begin_projected_application_read_attempt(current_admission, current_projection)
        .unwrap();
    let effects = reads
        .complete_projected_dependencies()
        .unwrap()
        .begin_effect_program();
    effects
        .projected_entity(&current_target)
        .expect("the target minted by this exact admission must remain usable");
    let denial = match effects.projected_entity(&prior_target) {
        Ok(_) => panic!("a target from another admission must not open an effect handle"),
        Err(denial) => denial,
    };
    assert_eq!(
        denial.kind(),
        primary_graph::WorthQueryApplicationAttemptDenialKind::ForeignEffectTarget
    );
}
