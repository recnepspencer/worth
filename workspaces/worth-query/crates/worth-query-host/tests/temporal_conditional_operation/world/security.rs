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
            .with_host_advancement(|phase| {
                self.application
                    .on_branch(self.application.current_world())
                    .select(&phase)
                    .expect("the selected product branch remains admitted")
                    .resolve_authenticated_principal(
                        &phase,
                        &binding,
                        &external,
                        &request,
                        primary_graph::WorthQueryPrincipalResolutionMode::Ordinary,
                    )
                    .unwrap()
            })
            .expect("installed host call opens");
        let mapping = self
            .application
            .with_host_advancement(|phase| {
                self.application
                    .on_branch(self.application.current_world())
                    .select(&phase)
                    .expect("the selected product branch remains admitted")
                    .resolve_entity(
                        &phase,
                        ExternalIdentityField::reference(),
                        declaration::authentication::WorthQueryExternalPrincipalIdentity::new(
                            "https://issuer.example",
                            "temporal-host",
                        )
                        .unwrap(),
                        &request,
                        primary_graph::WorthQueryPrincipalResolutionMode::Ordinary,
                    )
                    .unwrap()
            })
            .expect("installed host call opens");
        let operation = schema
            .installed_operation(RevokeTemporalPrincipal::reference())
            .unwrap();
        let admission = self
            .application
            .with_host_advancement(|phase| {
                self.application
                    .on_branch(self.application.current_world())
                    .select(&phase)
                    .unwrap()
                    .authorize_operation(
                        &phase,
                        &principal,
                        &mapping,
                        &operation,
                        Default::default(),
                        &request,
                    )
                    .unwrap()
            })
            .expect("installed host call opens");
        let (target, projection, _) = self
            .application
            .with_host_advancement(|phase| {
                self
            .invariant
            .project_admitted_operation(
                &phase, &admission,
                |reader, mapping| {
                    reader
                        .decision_field(mapping, MappingStatusField::reference())
                        .expect("revocation admits the mapping status decision field")
                        .expect("an authenticated mapping retains its status");
                    reader
                        .mutation_target(mapping)
                        .expect("the admitted mapping is a local mutation target")
                },
                worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
            )
            .unwrap()
            .into_parts()
            })
            .expect("installed host call opens");
        let mut effects = self
            .application
            .with_host_advancement(|phase| {
                let reads = self
                    .application
                    .begin_projected_application_read_attempt(
                        &phase,
                        admission,
                        projection,
                        worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
                    )
                    .unwrap();
                reads
                    .complete_projected_dependencies(
                        &phase,
                        worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
                    )
                    .unwrap()
                    .begin_effect_program()
            })
            .expect("installed host call opens");
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
            primary_graph::WorthQueryApplicationIdempotencyBinding::for_host_commit::<
                TemporalHostSchema,
                RevokeTemporalPrincipal,
                _,
                _,
            >("security", "security")
            .unwrap(),
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
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
            $world
                .application
                .with_host_advancement(|phase| {
                    let schema = $world.application.installed_schema();
                    let binding = schema
                        .principal_binding(TemporalPrincipalBinding::reference())
                        .unwrap();
                    let request = request_scope();
                    let external =
                        block_on(admit_identity_adapter(schema).authenticate((), &request))
                            .unwrap();
                    let principal = $world
                        .application
                        .on_branch($world.application.current_world())
                        .select(&phase)
                        .unwrap()
                        .resolve_authenticated_principal(
                            &phase,
                            &binding,
                            &external,
                            &request,
                            primary_graph::WorthQueryPrincipalResolutionMode::Ordinary,
                        )
                        .unwrap();
                    let mapping = $world
                        .application
                        .on_branch($world.application.current_world())
                        .select(&phase)
                        .unwrap()
                        .resolve_entity(
                            &phase,
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
                        .select(&phase)
                        .unwrap()
                        .authorize_operation(
                            &phase,
                            &principal,
                            &mapping,
                            &operation,
                            Default::default(),
                            &request,
                        )
                        .unwrap()
                })
                .expect("installed host call opens")
        }};
    }

    let world = CourtroomWorld::publish("ready");
    let prior_admission = admit_revocation!(world);
    let (prior_target, prior_projection, _) = world
        .application
        .with_host_advancement(|phase| {
            world
                .invariant
                .project_admitted_operation(
                    &phase,
                    &prior_admission,
                    |reader, mapping| {
                        reader
                            .decision_field(mapping, MappingStatusField::reference())
                            .unwrap();
                        reader.mutation_target(mapping).unwrap()
                    },
                    worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
                )
                .unwrap()
                .into_parts()
        })
        .expect("installed host call opens");
    drop(prior_projection);

    let current_admission = admit_revocation!(world);
    let (current_target, current_projection, _) = world
        .application
        .with_host_advancement(|phase| {
            world
                .invariant
                .project_admitted_operation(
                    &phase,
                    &current_admission,
                    |reader, mapping| {
                        reader
                            .decision_field(mapping, MappingStatusField::reference())
                            .unwrap();
                        reader.mutation_target(mapping).unwrap()
                    },
                    worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
                )
                .unwrap()
                .into_parts()
        })
        .expect("installed host call opens");
    let effects = world
        .application
        .with_host_advancement(|phase| {
            let reads = world
                .application
                .begin_projected_application_read_attempt(
                    &phase,
                    current_admission,
                    current_projection,
                    worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
                )
                .unwrap();
            reads
                .complete_projected_dependencies(
                    &phase,
                    worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
                )
                .unwrap()
                .begin_effect_program()
        })
        .expect("installed host call opens");
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
