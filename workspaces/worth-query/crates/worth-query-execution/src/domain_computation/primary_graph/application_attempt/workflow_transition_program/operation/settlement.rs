//! Settlement consumes a validated performed receipt, never a proposed input.
use super::*;

impl<Schema, Operation, Input, Scope> PreparedWorkflowOperation<Schema, Operation, Input, Scope>
where
    Schema: ApplicationSchema,
    Operation: 'static,
{
    pub(in crate::domain_computation::primary_graph::application_attempt::workflow_transition_program) fn settle<
        Binding,
    >(
        self,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        receipt: &WorthQueryApplicationCommitReceipt,
    ) -> Result<
        PreparedWorkflowAdvance<Schema, Operation, Input, Scope>,
        WorthQueryApplicationAttemptDenial,
    >
    where
        Binding: ApplicationMutationBinding<Schema>,
    {
        let performed = validate_operation_receipt::<Schema, Binding>(
            runtime,
            &self.required,
            self.admitted.subject(),
            self.admitted.read_set().lease.product().product_branch(),
            receipt,
            None,
        )?;
        self.settle_validated(performed)
    }

    pub(in crate::domain_computation::primary_graph::application_attempt::workflow_transition_program) fn settle_recovered<
        Binding,
    >(
        self,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        receipt: &WorthQueryApplicationCommitReceipt,
        recovery: &crate::domain_computation::application_aftermath::WorthQueryRecoverySafeRetryAdmission,
    ) -> Result<
        PreparedWorkflowAdvance<Schema, Operation, Input, Scope>,
        WorthQueryApplicationAttemptDenial,
    >
    where
        Binding: ApplicationMutationBinding<Schema>,
    {
        let performed = validate_operation_receipt::<Schema, Binding>(
            runtime,
            &self.required,
            self.admitted.subject(),
            self.admitted.read_set().lease.product().product_branch(),
            receipt,
            Some(recovery),
        )?;
        self.settle_validated(performed)
    }

    fn settle_validated(
        self,
        performed: ValidatedWorkflowOperationReceipt<'_>,
    ) -> Result<
        PreparedWorkflowAdvance<Schema, Operation, Input, Scope>,
        WorthQueryApplicationAttemptDenial,
    > {
        let receipt_identity = performed.identity();
        let local_origin =
            self.required.local_action.is_some().then(|| {
                super::LocalActionSettlementProof::from_performed(&self.required, &performed)
            });
        let executor = crate::domain_computation::primary_graph::workflow::actor::record::WorkflowAdmittedActor::from_performed_receipt(&performed);
        let mut effects = Vec::new();
        let basis = match self.required.local_action_kind() {
            Some(super::super::WorkflowLocalActionKind::Return) => self
                .admitted
                .operation_settlement_basis()
                .for_business_return(),
            Some(super::super::WorkflowLocalActionKind::Terminate) => self
                .admitted
                .operation_settlement_basis()
                .for_business_termination(),
            None => self.admitted.operation_settlement_basis(),
        };
        let terminal = self.required.local_action_kind()
            == Some(super::super::WorkflowLocalActionKind::Terminate);
        let transition = crate::domain_computation::primary_graph::workflow::instance::visit_workflow_operation_settlement_facts(
            &self.layout,
            &basis,
            &receipt_identity,
            executor,
            |effect| {
                effects.push(effect);
                Ok::<(), WorthQueryApplicationAttemptDenial>(())
            },
        )?;
        crate::domain_computation::primary_graph::workflow::operation_output::append_operation_outputs(
            &self.layout, transition, self.required.binding().expect("validated operation binding"),
            performed.outputs().native_witness_roles().map(|(role, posture, entity, target)|
                (role, posture, entity, worth_relational::facade::transactions::EntityReference::Existing(target))),
            &mut effects,
        );
        let mut demand = super::super::PlatformEffectDemand::default();
        for effect in &effects {
            demand.observe(effect)?;
        }
        let reservation = if self.required.local_action.is_some() {
            super::super::super::effect_program::admit_workflow_settlement_effects(
                &self.admitted.read_set().admission,
                demand,
            )?
        } else {
            super::super::admit_platform_effects(self.admitted.read_set(), demand)?
        };
        reservation.materialize(&effects)?;
        let transition_identity = self.admitted.identity().to_owned();
        let transition_identity_bytes = *self.admitted.identity_bytes();
        let instance = self.admitted.instance();
        let node_path = self.admitted.node_path().to_owned();
        let progress_update = if terminal {
            None
        } else {
            Some(basis.prepare_progress_update(receipt_identity)?)
        };
        let mut read_set = self.admitted.into_read_set();
        read_set.local_action_settlement = local_origin;
        let program = WorthQueryApplicationEffectProgram {
            read_set,
            effects,
            emission_retained_bytes: 0,
            emission_retained_bytes_ceiling: 0,
            conditional_definition: None,
            effect_posture: crate::domain_computation::provider_session::WorthQueryApplicationEffectPosture::Platform,
            output_correspondence: Default::default(),
            retain_output_demand_observation: false,
            retain_client_observation: false,
            producer_required_invariants: &[],
            output_currentness_facts: None,
        };
        Ok(PreparedWorkflowAdvance::Transition {
            program,
            program_revision: self.program_revision,
            transition_identity,
            transition_identity_bytes,
            transition_identity_locator: self.layout.transition.identity.clone(),
            assessment_identity_locator: self.layout.assessment_evidence.identity.clone(),
            instance,
            node_path,
            assessment: None,
            supporting_identity: Some(receipt_identity),
            operation_receipt_identity: Some(receipt_identity),
            progress_update,
            terminal,
            approval: None,
            approval_identity: None,
            approval_authentication: None,
            replays: self.replays,
        })
    }
}
