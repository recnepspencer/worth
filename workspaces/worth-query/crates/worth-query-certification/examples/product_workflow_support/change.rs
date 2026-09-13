use std::sync::Arc;

use worth_query_host::facade::{primary_graph, product};

use super::adapters::{block_on, ReplacementPredicate};
use super::application::{admit_identity_adapter, ExampleApplication};
use super::schema::*;

pub type ExampleRelationalChange = product::WorthQueryAdmittedChange<
    TemporalHostSchema,
    AmendTemporal,
    AmendTemporalInput,
    TemporalIntent,
>;

pub type ExampleCombinedChange = product::WorthQueryAdmittedChange<
    TemporalHostSchema,
    AmendTemporalAndPublishDefinition,
    AmendTemporalInput,
    TemporalIntent,
>;

impl ExampleApplication {
    pub fn admit_input_change(
        &self,
        selected: primary_graph::WorthQuerySelectedProductOperation<'_, TemporalHostSchema>,
        input: &str,
        idempotency_seed: u8,
    ) -> ExampleRelationalChange {
        let schema = self.runtime.installed_schema();
        let principal_binding = schema
            .principal_binding(TemporalPrincipalBinding::reference())
            .expect("the principal binding must be installed");
        let authentication = admit_identity_adapter(schema);
        let request = super::adapters::request_scope();
        let external = block_on(authentication.authenticate((), &request))
            .expect("the example identity must authenticate");
        let principal = selected
            .resolve_authenticated_principal(
                &principal_binding,
                &external,
                &request,
                primary_graph::WorthQueryPrincipalResolutionMode::Ordinary,
            )
            .expect("the selected product must resolve the principal");
        let intent = selected
            .resolve_entity(
                IntentIdentityField::reference(),
                "intent-1".to_owned(),
                &request,
                primary_graph::WorthQueryPrincipalResolutionMode::Ordinary,
            )
            .expect("the selected product must resolve the intent");
        let operation = schema
            .installed_operation(AmendTemporal::reference())
            .expect("the input change operation must be installed");
        let admission = selected
            .authorize_operation(
                &principal,
                &intent,
                &operation,
                Default::default(),
                &request,
            )
            .expect("the selected product change must be authorized");
        let (_, projection, _) = self
            .invariant
            .project_admitted_operation(&admission, |reader, scope| {
                reader
                    .decision_field(scope, IntentRevisionField::reference())
                    .expect("revision must project");
                reader
                    .decision_field(scope, IntentLifecycleField::reference())
                    .expect("lifecycle must project");
                reader
                    .decision_field(scope, IntentGateField::reference())
                    .expect("gate must project");
                reader
                    .decision_field(scope, IntentDueField::reference())
                    .expect("due coordinate must project");
                reader
                    .decision_field(scope, IntentInputField::reference())
                    .expect("input must project");
            })
            .expect("the admitted change must project")
            .into_parts();
        let reads = self
            .runtime
            .begin_projected_application_read_attempt(admission, projection)
            .expect("the projected change must begin");
        let mut effects = reads
            .complete_projected_dependencies()
            .expect("the exact dependencies must remain complete")
            .begin_effect_program();
        let intent = effects
            .existing_entity(&intent)
            .expect("the admitted intent must enter the effect program");
        effects
            .write_field(&intent, IntentRevisionField::reference(), 2)
            .expect("revision write must be admitted");
        effects
            .write_field(
                &intent,
                IntentLifecycleField::reference(),
                "active".to_owned(),
            )
            .expect("lifecycle write must be admitted");
        effects
            .write_field(&intent, IntentDueField::reference(), 11)
            .expect("due-coordinate write must be admitted");
        effects
            .write_field(&intent, IntentInputField::reference(), input.to_owned())
            .expect("input write must be admitted");
        effects
            .write_field(&intent, IntentGateField::reference(), "ready".to_owned())
            .expect("gate write must be admitted");
        product::WorthQueryAdmittedChange::new(
            effects.finish().expect("the change program must finish"),
            primary_graph::WorthQueryApplicationIdempotencyBinding::new(
                [idempotency_seed; 32],
                [idempotency_seed.wrapping_add(0x10); 32],
            ),
        )
    }

    pub fn admit_combined_change(
        &self,
        selected: primary_graph::WorthQuerySelectedProductOperation<'_, TemporalHostSchema>,
        input: &str,
        provider: Arc<ReplacementPredicate>,
    ) -> ExampleCombinedChange {
        let schema = self.runtime.installed_schema();
        let principal_binding = schema
            .principal_binding(TemporalPrincipalBinding::reference())
            .expect("the principal binding must be installed");
        let authentication = admit_identity_adapter(schema);
        let request = super::adapters::request_scope();
        let external = block_on(authentication.authenticate((), &request))
            .expect("the example identity must authenticate");
        let conditional_definition = selected
            .admit_application_conditional_definition(&self.clock, provider)
            .expect("the replacement conditional definition must be admitted");
        let principal = selected
            .resolve_authenticated_principal(
                &principal_binding,
                &external,
                &request,
                primary_graph::WorthQueryPrincipalResolutionMode::Ordinary,
            )
            .expect("the selected product must resolve the principal");
        let intent = selected
            .resolve_entity(
                IntentIdentityField::reference(),
                "intent-1".to_owned(),
                &request,
                primary_graph::WorthQueryPrincipalResolutionMode::Ordinary,
            )
            .expect("the selected product must resolve the intent");
        let operation = schema
            .installed_operation(AmendTemporalAndPublishDefinition::reference())
            .expect("the combined change operation must be installed");
        let admission = selected
            .authorize_operation(
                &principal,
                &intent,
                &operation,
                Default::default(),
                &request,
            )
            .expect("the selected product combined change must be authorized");
        let (_, projection, _) = self
            .invariant
            .project_admitted_operation(&admission, |reader, scope| {
                reader
                    .decision_field(scope, IntentRevisionField::reference())
                    .expect("revision must project");
                reader
                    .decision_field(scope, IntentLifecycleField::reference())
                    .expect("lifecycle must project");
                reader
                    .decision_field(scope, IntentGateField::reference())
                    .expect("gate must project");
                reader
                    .decision_field(scope, IntentDueField::reference())
                    .expect("due coordinate must project");
                reader
                    .decision_field(scope, IntentInputField::reference())
                    .expect("input must project");
            })
            .expect("the admitted combined change must project")
            .into_parts();
        let reads = self
            .runtime
            .begin_projected_application_read_attempt(admission, projection)
            .expect("the projected combined change must begin");
        let mut effects = reads
            .complete_projected_dependencies()
            .expect("the exact dependencies must remain complete")
            .begin_effect_program();
        let intent = effects
            .existing_entity(&intent)
            .expect("the admitted intent must enter the effect program");
        effects
            .write_field(&intent, IntentRevisionField::reference(), 2)
            .expect("revision write must be admitted");
        effects
            .write_field(
                &intent,
                IntentLifecycleField::reference(),
                "active".to_owned(),
            )
            .expect("lifecycle write must be admitted");
        effects
            .write_field(&intent, IntentDueField::reference(), 11)
            .expect("due-coordinate write must be admitted");
        effects
            .write_field(&intent, IntentInputField::reference(), input.to_owned())
            .expect("input write must be admitted");
        effects
            .write_field(&intent, IntentGateField::reference(), "ready".to_owned())
            .expect("gate write must be admitted");
        effects
            .advance_conditional_definition(conditional_definition)
            .expect("the admitted definition must join the effect program");
        effects
            .emit_external(
                TemporalAmendmentEffect::reference(),
                TemporalAmendmentNotice(input.to_owned()),
            )
            .expect("the installed external effect must join the program");
        product::WorthQueryAdmittedChange::new(
            effects
                .finish()
                .expect("the combined change program must finish"),
            primary_graph::WorthQueryApplicationIdempotencyBinding::new([0x51; 32], [0x61; 32]),
        )
    }
}
