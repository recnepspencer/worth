use worth_query_host::facade::{primary_graph, product};

use super::super::adapters::block_on;
use super::super::schema::*;
use super::{admit_identity_adapter, request_scope, CourtroomWorld};

impl CourtroomWorld {
    #[allow(dead_code)] // Shared by the certification crate's cost targets.
    pub fn change_inputs_on_branch(
        &self,
        branch: product::WorthQueryProductBranch,
        touched_records: usize,
    ) -> primary_graph::WorthQueryApplicationCommitReceipt {
        let selected = self.application.on_branch(branch).select().unwrap();
        let schema = self.application.installed_schema();
        let request = request_scope();
        let targets = (1..=touched_records)
            .map(|ordinal| {
                selected
                    .resolve_entity(
                        IntentIdentityField::reference(),
                        format!("intent-{ordinal}"),
                        &request,
                        primary_graph::WorthQueryPrincipalResolutionMode::Ordinary,
                    )
                    .expect("each populated intent must remain resolvable")
            })
            .collect::<Vec<_>>();
        let scope = targets
            .first()
            .expect("a scaled amendment must touch at least one record");
        let principal_binding = schema
            .principal_binding(TemporalPrincipalBinding::reference())
            .unwrap();
        let authentication = admit_identity_adapter(schema);
        let external = block_on(authentication.authenticate((), &request)).unwrap();
        let principal = selected
            .resolve_authenticated_principal(
                &principal_binding,
                external,
                &request,
                primary_graph::WorthQueryPrincipalResolutionMode::Ordinary,
            )
            .unwrap();
        let operation = schema
            .installed_operation(AmendTemporal::reference())
            .unwrap();
        let admission = selected
            .authorize_operation(&principal, scope, &operation, Default::default(), &request)
            .unwrap();
        let (_, projection, _) = self
            .invariant
            .project_admitted_operation(&admission, |reader, _| {
                for ordinal in 1..=touched_records {
                    let target = reader
                        .resolve_entity(
                            IntentIdentityField::reference(),
                            format!("intent-{ordinal}"),
                        )
                        .unwrap();
                    reader
                        .decision_field(&target, IntentInputField::reference())
                        .unwrap();
                }
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
        for (ordinal, target) in targets.iter().enumerate() {
            let target = effects.existing_entity(target).unwrap();
            effects
                .write_field(
                    &target,
                    IntentInputField::reference(),
                    format!("scaled-input-{}", ordinal + 1),
                )
                .unwrap();
        }
        let change = product::WorthQueryAdmittedChange::new(
            effects.finish().unwrap(),
            primary_graph::WorthQueryApplicationIdempotencyBinding::new(
                [0x63 ^ touched_records as u8; 32],
                [0x36 ^ touched_records as u8; 32],
            ),
        );
        self.application
            .on_branch(branch)
            .transaction()
            .apply(change)
            .commit()
            .expect("the scaled change remains on its admitted product")
            .require_committed()
            .expect("the scaled public amendment must commit")
    }
}
