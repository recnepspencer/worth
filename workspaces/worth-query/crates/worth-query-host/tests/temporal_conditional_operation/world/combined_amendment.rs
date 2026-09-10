use worth_query_host::facade::{primary_graph, product};

use super::super::adapters::{block_on, ReplacementPredicate};
use super::super::schema::*;
use super::{admit_identity_adapter, request_scope, CourtroomWorld};

impl CourtroomWorld {
    pub fn change_input_and_conditional_definition_on_branch(
        &self,
        branch: product::WorthQueryProductBranch,
        input: &str,
        provider: std::sync::Arc<ReplacementPredicate>,
    ) -> primary_graph::WorthQueryApplicationCommitReceipt {
        let selected = self.application.on_branch(branch).select().unwrap();
        self.change_input_and_conditional_definition_on_selected(selected, input, provider)
    }

    fn change_input_and_conditional_definition_on_selected(
        &self,
        selected: primary_graph::WorthQuerySelectedProductOperation<'_, TemporalHostSchema>,
        input: &str,
        provider: std::sync::Arc<ReplacementPredicate>,
    ) -> primary_graph::WorthQueryApplicationCommitReceipt {
        let schema = self.application.installed_schema();
        let principal_binding = schema
            .principal_binding(TemporalPrincipalBinding::reference())
            .unwrap();
        let authentication = admit_identity_adapter(schema);
        let request = request_scope();
        let external = block_on(authentication.authenticate((), &request)).unwrap();
        let branch = selected.product().product_branch();
        let conditional_definition = selected
            .admit_application_conditional_definition(&self.clock, provider)
            .unwrap();
        let principal = selected
            .resolve_authenticated_principal(
                &principal_binding,
                external,
                &request,
                primary_graph::WorthQueryPrincipalResolutionMode::Ordinary,
            )
            .unwrap();
        let intent = selected
            .resolve_entity(
                IntentIdentityField::reference(),
                "intent-1".to_string(),
                &request,
                primary_graph::WorthQueryPrincipalResolutionMode::Ordinary,
            )
            .unwrap();
        let operation = schema
            .installed_operation(AmendTemporalAndPublishDefinition::reference())
            .unwrap();
        let admission = selected
            .authorize_operation(
                &principal,
                &intent,
                &operation,
                Default::default(),
                &request,
            )
            .unwrap();
        let (_, projection, _) = self
            .invariant
            .project_admitted_operation(&admission, |reader, scope| {
                reader
                    .decision_field(scope, IntentRevisionField::reference())
                    .unwrap();
                reader
                    .decision_field(scope, IntentLifecycleField::reference())
                    .unwrap();
                reader
                    .decision_field(scope, IntentGateField::reference())
                    .unwrap();
                reader
                    .decision_field(scope, IntentDueField::reference())
                    .unwrap();
                reader
                    .decision_field(scope, IntentInputField::reference())
                    .unwrap();
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
        let intent = effects.existing_entity(&intent).unwrap();
        effects
            .write_field(&intent, IntentRevisionField::reference(), 2)
            .unwrap();
        effects
            .write_field(
                &intent,
                IntentLifecycleField::reference(),
                "active".to_string(),
            )
            .unwrap();
        effects
            .write_field(&intent, IntentDueField::reference(), 11)
            .unwrap();
        effects
            .write_field(&intent, IntentInputField::reference(), input.to_string())
            .unwrap();
        effects
            .write_field(&intent, IntentGateField::reference(), "ready".to_string())
            .unwrap();
        effects
            .advance_conditional_definition(conditional_definition)
            .unwrap();
        effects
            .emit_external(
                TemporalAmendmentEffect::reference(),
                TemporalAmendmentNotice(input.to_string()),
            )
            .unwrap();
        let change = product::WorthQueryAdmittedChange::new(
            effects.finish().unwrap(),
            primary_graph::WorthQueryApplicationIdempotencyBinding::new([0x7A; 32], [0x4B; 32]),
        );
        self.application
            .on_branch(branch)
            .transaction()
            .apply(change)
            .commit()
            .expect("the admitted combined change remains on its selected product")
            .require_committed()
            .expect("the combined amendment must commit")
    }
}
