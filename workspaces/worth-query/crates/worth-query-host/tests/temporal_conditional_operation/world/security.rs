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
