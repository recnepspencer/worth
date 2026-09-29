impl super::WorthUiActiveApplicationSession {
    /// The owners an activation observation against `prepared` reads at
    /// `generation`.
    pub(in crate::facade::entry) fn intent_read_owners<'owners>(
        &'owners self,
        prepared: &'owners crate::facade::prepared_application_authority::WorthUiPreparedApplicationAuthority,
        generation: &'owners crate::runtime::WorthUiActiveApplicationGenerationIdentity,
    ) -> crate::runtime::intent::UiIntentOperabilityReadOwners<'owners> {
        crate::runtime::intent::UiIntentOperabilityReadOwners {
            authority: crate::runtime::intent::UiIntentOperabilityAuthority {
                catalog: prepared.intent_catalog(),
                definitions: prepared.capabilities().intent_definitions(),
                execution_bindings: prepared.intent_execution_bindings(),
                occupancy: self.intent_execution.occupancy(),
            },
            generation,
            inputs: crate::runtime::intent::UiIntentInputOwners {
                mounted: &self.mounted,
                application_facts: &self.intent_application_facts,
                expressions: &self.expressions,
            },
        }
    }

    /// The owners an activation observation at `successor` reads before that
    /// generation is active: those of [`Self::intent_read_owners`], with
    /// conditions read through the prepared expression succession.
    fn successor_read_owners<'owners>(
        &'owners self,
        prepared: &'owners crate::facade::prepared_application_authority::WorthUiPreparedApplicationAuthority,
        successor: &'owners crate::runtime::WorthUiActiveApplicationGenerationIdentity,
        expressions: &'owners crate::runtime::expression::UiPreparedExpressionSuccession,
    ) -> crate::runtime::intent::UiIntentOperabilityReadOwners<'owners> {
        let mut owners = self.intent_read_owners(prepared, successor);
        owners.inputs.expressions = expressions.successor_owner(&self.expressions);
        owners
    }

    pub(in crate::facade::entry) fn include_pointer_publication(
        &self,
        plan: &mut crate::runtime::rebind::UiRebindPlan,
        pointer: &crate::runtime::pointer_affordance::UiPreparedPointerAffordanceGenerationSuccession,
    ) -> Result<(), crate::runtime::rebind::UiRebindPreparationDenial> {
        let mut targets = Vec::new();
        for target in pointer.affected_targets() {
            let basis = self.mounted.current_mounted_identity_basis(target).ok_or(
                crate::runtime::rebind::UiRebindPreparationDenial::CandidateBindingMismatch,
            )?;
            let node = self
                .application
                .prepared_authority()
                .graph_snapshot()
                .nodes()
                .iter()
                .find(|node| node.graph_node_identity() == basis.graph_node_identity())
                .ok_or(
                    crate::runtime::rebind::UiRebindPreparationDenial::CandidateBindingMismatch,
                )?;
            targets.push(crate::runtime::rebind::UiRebindPlanTarget::Consumer(
                crate::graph::UiGraphFactConsumerKey::new(
                    crate::graph::UiGraphFactConsumerKind::GraphNode,
                    node.declaration_identity().authored_semantic_name(),
                    node.repeated_instance_basis().identity_digest(),
                ),
            ));
        }
        plan.include_surface_consumers(targets);
        Ok(())
    }

    pub(in crate::facade::entry) fn prepare_pointer_generation_succession(
        &self,
        prepared: &crate::facade::prepared_application_authority::WorthUiPreparedApplicationAuthority,
        expressions: &crate::runtime::expression::UiPreparedExpressionSuccession,
    ) -> Result<
        crate::runtime::pointer_affordance::UiPreparedPointerAffordanceGenerationSuccession,
        crate::runtime::rebind::UiRebindPreparationDenial,
    > {
        let successor = crate::runtime::WorthUiActiveApplicationGenerationIdentity::current(
            self.session_identity(),
            prepared.generation_identity(),
        );
        let now = self
            .observation_clock
            .as_ref()
            .map(|clock| clock.sample_millis());
        let succession = crate::facade::prepared_application_authority::WorthUiPreparedApplicationGenerationSuccession::new(
            self.generation_identity().clone(), prepared.generation_identity().clone());
        crate::runtime::pointer_affordance::UiPreparedPointerAffordanceGenerationSuccession::prepare(
            self.active_generation_identity(), successor.clone(), &succession,
            prepared.capabilities().digest().as_u64(),
            self.pointer_affordance_snapshot.as_ref(), now, &self.mounted,
            |target| crate::runtime::intent::observe_activation_operability(
                target, self.successor_read_owners(prepared, &successor, expressions), &self.intent_confirmation,
                now.map(worth_ui_host_contract::UiHostObservationTimeBasis::HostMonotonicMillis),
            ),
        ).map_err(|_| crate::runtime::rebind::UiRebindPreparationDenial::CandidateCutoverPreparation)
    }

    pub(in crate::facade::entry) fn prepare_pointer_graph_succession(
        &self,
        succession: &crate::facade::prepared_application_authority::WorthUiPreparedApplicationGenerationSuccession,
        expressions: &crate::runtime::expression::UiPreparedExpressionSuccession,
    ) -> Result<
        crate::runtime::pointer_affordance::UiPreparedPointerAffordanceGenerationSuccession,
        (),
    > {
        let successor = crate::runtime::WorthUiActiveApplicationGenerationIdentity::current(
            self.session_identity(),
            succession.successor(),
        );
        let prepared = self.application.prepared_authority();
        let now = self
            .observation_clock
            .as_ref()
            .map(|clock| clock.sample_millis());
        crate::runtime::pointer_affordance::UiPreparedPointerAffordanceGenerationSuccession::prepare(
            self.active_generation_identity(), successor.clone(), succession,
            prepared.capabilities().digest().as_u64(), self.pointer_affordance_snapshot.as_ref(), now, &self.mounted,
            |target| crate::runtime::intent::observe_activation_operability(
                target, self.successor_read_owners(prepared, &successor, expressions), &self.intent_confirmation,
                now.map(worth_ui_host_contract::UiHostObservationTimeBasis::HostMonotonicMillis),
            ),
        ).map_err(|_| ())
    }
}
