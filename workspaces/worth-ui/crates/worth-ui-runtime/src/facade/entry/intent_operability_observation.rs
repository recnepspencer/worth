impl super::WorthUiActiveApplicationSession {
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
                target, prepared.intent_catalog(), prepared.capabilities().intent_definitions(),
                prepared.intent_execution_bindings(), &successor, &self.mounted,
                &self.intent_application_facts, self.intent_execution.occupancy(),
                &self.intent_confirmation,
                now.map(worth_ui_host_contract::UiHostObservationTimeBasis::HostMonotonicMillis),
            ),
        ).map_err(|_| crate::runtime::rebind::UiRebindPreparationDenial::CandidateCutoverPreparation)
    }

    pub(in crate::facade::entry) fn prepare_pointer_graph_succession(
        &self,
        succession: &crate::facade::prepared_application_authority::WorthUiPreparedApplicationGenerationSuccession,
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
                target, prepared.intent_catalog(), prepared.capabilities().intent_definitions(),
                prepared.intent_execution_bindings(), &successor, &self.mounted,
                &self.intent_application_facts, self.intent_execution.occupancy(), &self.intent_confirmation,
                now.map(worth_ui_host_contract::UiHostObservationTimeBasis::HostMonotonicMillis),
            ),
        ).map_err(|_| ())
    }
}
