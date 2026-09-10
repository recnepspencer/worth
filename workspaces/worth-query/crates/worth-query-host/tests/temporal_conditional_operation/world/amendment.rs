use worth_query_host::facade::{primary_graph, product};

use super::super::adapters::block_on;
use super::super::schema::*;
use super::{admit_identity_adapter, request_scope, CourtroomWorld};

enum AmendmentWidth {
    GateOnly,
    Full,
}

impl CourtroomWorld {
    pub fn amend_intent(&mut self, revision: u64, lifecycle: &str, gate: &str) {
        self.supersede_intent(revision, 5, lifecycle, "payload", gate);
    }

    pub fn amend_gate_only(&mut self, gate: &str) {
        self.amendment_ordinal += 1;
        let branch = self.application.current_world();
        self.commit_amendment(
            branch,
            1,
            5,
            "active",
            "payload",
            gate,
            AmendmentWidth::GateOnly,
            self.amendment_ordinal,
        );
    }

    pub fn intent_record_identity(&self) -> primary_graph::RelationalBridgeRecordIdentityParts {
        self.application
            .on_branch(self.application.current_world())
            .select()
            .expect("the selected product branch remains admitted")
            .resolve_entity(
                IntentIdentityField::reference(),
                "intent-1".to_string(),
                &request_scope(),
                primary_graph::WorthQueryPrincipalResolutionMode::Certification,
            )
            .expect("the courtroom intent must remain exactly resolvable")
            .relational_record_identity_parts()
    }

    pub fn supersede_intent(
        &mut self,
        revision: u64,
        due: u64,
        lifecycle: &str,
        input: &str,
        gate: &str,
    ) {
        self.amendment_ordinal += 1;
        let branch = self.application.current_world();
        self.commit_amendment(
            branch,
            revision,
            due,
            lifecycle,
            input,
            gate,
            AmendmentWidth::Full,
            self.amendment_ordinal,
        );
    }

    pub fn change_input_after_query_admission(&self, input: &str) {
        let branch = self.application.current_world();
        self.commit_amendment(
            branch,
            2,
            11,
            "active",
            input,
            "ready",
            AmendmentWidth::Full,
            0xED,
        );
    }

    pub fn retry_input_change_on_branch(
        &self,
        branch: product::WorthQueryProductBranch,
        input: &str,
    ) -> primary_graph::WorthQueryApplicationCommitOutcome {
        self.compare_amendment(
            branch,
            2,
            11,
            "active",
            input,
            "ready",
            AmendmentWidth::Full,
            0xEC,
        )
    }

    pub fn change_input_on_branch(
        &self,
        branch: product::WorthQueryProductBranch,
        input: &str,
    ) -> primary_graph::WorthQueryApplicationCommitOutcome {
        let selected = self.application.on_branch(branch).select().unwrap();
        self.compare_amendment_program(
            selected,
            2,
            11,
            "active",
            input,
            "ready",
            AmendmentWidth::Full,
            0xEC,
        )
    }

    fn commit_amendment(
        &self,
        branch: product::WorthQueryProductBranch,
        revision: u64,
        due: u64,
        lifecycle: &str,
        input: &str,
        gate: &str,
        width: AmendmentWidth,
        amendment_ordinal: u8,
    ) -> primary_graph::WorthQueryApplicationCommitReceipt {
        let outcome = self.compare_amendment(
            branch,
            revision,
            due,
            lifecycle,
            input,
            gate,
            width,
            amendment_ordinal,
        );
        let primary_graph::WorthQueryApplicationCommitOutcome::Committed(receipt) = outcome else {
            panic!("unexpected amendment outcome: {outcome:?}")
        };
        receipt
    }

    fn compare_amendment(
        &self,
        branch: product::WorthQueryProductBranch,
        revision: u64,
        due: u64,
        lifecycle: &str,
        input: &str,
        gate: &str,
        width: AmendmentWidth,
        amendment_ordinal: u8,
    ) -> primary_graph::WorthQueryApplicationCommitOutcome {
        let selected = self.application.on_branch(branch).select().unwrap();
        self.compare_amendment_program(
            selected,
            revision,
            due,
            lifecycle,
            input,
            gate,
            width,
            amendment_ordinal,
        )
    }

    fn compare_amendment_program(
        &self,
        selected: primary_graph::WorthQuerySelectedProductOperation<'_, TemporalHostSchema>,
        revision: u64,
        due: u64,
        lifecycle: &str,
        input: &str,
        gate: &str,
        width: AmendmentWidth,
        amendment_ordinal: u8,
    ) -> primary_graph::WorthQueryApplicationCommitOutcome {
        let schema = self.application.installed_schema();
        let principal_binding = schema
            .principal_binding(TemporalPrincipalBinding::reference())
            .unwrap();
        let authentication = admit_identity_adapter(schema);
        let request = request_scope();
        let external = block_on(authentication.authenticate((), &request)).unwrap();
        let branch = selected.product().product_branch();
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
            .installed_operation(AmendTemporal::reference())
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
        if matches!(width, AmendmentWidth::Full) {
            effects
                .write_field(&intent, IntentRevisionField::reference(), revision)
                .unwrap();
            effects
                .write_field(
                    &intent,
                    IntentLifecycleField::reference(),
                    lifecycle.to_string(),
                )
                .unwrap();
            effects
                .write_field(&intent, IntentDueField::reference(), due)
                .unwrap();
            effects
                .write_field(&intent, IntentInputField::reference(), input.to_string())
                .unwrap();
        }
        effects
            .write_field(&intent, IntentGateField::reference(), gate.to_string())
            .unwrap();
        let idempotency = primary_graph::WorthQueryApplicationIdempotencyBinding::new(
            [0x91 ^ amendment_ordinal; 32],
            [0xA0 ^ amendment_ordinal; 32],
        );
        let admitted_change =
            product::WorthQueryAdmittedChange::new(effects.finish().unwrap(), idempotency);
        self.application
            .on_branch(branch)
            .transaction()
            .apply(admitted_change)
            .commit()
            .expect("the admitted change and transaction select the same product occurrence")
    }
}
