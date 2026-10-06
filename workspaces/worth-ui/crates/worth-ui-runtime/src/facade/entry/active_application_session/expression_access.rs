impl super::WorthUiActiveApplicationSession {
    /// The retained evaluation record of the authored expression named
    /// `identity`, or `None` when the active application has no such
    /// expression.
    pub fn expression_record(
        &self,
        identity: &str,
    ) -> Option<&crate::facade::expression::UiExpressionEvaluationRecord> {
        self.expressions.record_by_identity(identity)
    }

    /// A reference to the retained result of the expression named `identity`,
    /// whatever its posture, for consumers that must later prove they used
    /// the current result.
    pub fn expression_result(
        &self,
        identity: &str,
    ) -> Option<crate::facade::expression::UiExpressionResultReference> {
        let slot = self.expressions.catalog().slot_of(identity)?;
        self.expressions.result(slot)
    }

    /// Whether `reference` still names the retained result of the active
    /// generation, with no evaluation since that changed the outcome.
    pub fn is_current_expression_result(
        &self,
        reference: &crate::facade::expression::UiExpressionResultReference,
    ) -> bool {
        self.expressions
            .is_current_result(reference, &self.active_generation_identity())
    }

    /// Prepares the expression owner's move to `successor`, against the
    /// catalog it installs, reading the mounted state and application facts
    /// as they stand now. No record changes until
    /// [`Self::follow_application_generation`] commits it.
    pub(in crate::facade::entry) fn prepare_expression_succession(
        &mut self,
        successor: &crate::runtime::WorthUiActiveApplicationGenerationIdentity,
        catalog: &std::sync::Arc<crate::runtime::expression::UiExpressionCatalog>,
    ) -> crate::runtime::expression::UiPreparedExpressionSuccession {
        self.expressions.prepare_succession(
            catalog,
            &crate::runtime::expression::UiExpressionInputs {
                generation: successor,
                mounted: &self.mounted,
                facts: &self.intent_application_facts,
            },
        )
    }

    /// Commits `prepared`, moving the expression owner to the active
    /// generation and to the catalog the prepared authority installs now.
    /// Every writer of the active generation commits here once the successor
    /// owners it reads are in place: an evidence-only rebind, an application
    /// cutover and the initial mounted establishment. An owner that missed
    /// one holds nothing current.
    pub(in crate::facade::entry) fn follow_application_generation(
        &mut self,
        prepared: crate::runtime::expression::UiPreparedExpressionSuccession,
    ) {
        let active = self.active_generation_identity();
        let settlement = self.expressions.commit_succession(
            prepared,
            self.application.prepared_authority().expression_catalog(),
            &crate::runtime::expression::UiExpressionInputs {
                generation: &active,
                mounted: &self.mounted,
                facts: &self.intent_application_facts,
            },
        );
        self.reobserve_condition_consumers(settlement);
    }

    /// Refreshes the standing facts of the declarations that read a condition
    /// `settlement` changed, against the active generation.
    pub(in crate::facade::entry) fn reobserve_condition_consumers(
        &mut self,
        settlement: crate::runtime::expression::UiExpressionSettlement,
    ) {
        let active = self.active_generation_identity();
        let prepared = self.application.prepared_authority();
        self.intent_admission.reobserve_condition_consumers(
            settlement,
            crate::runtime::intent::UiIntentOperabilityReadOwners {
                authority: crate::runtime::intent::UiIntentOperabilityAuthority {
                    catalog: prepared.intent_catalog(),
                    definitions: prepared.capabilities().intent_definitions(),
                    execution_bindings: prepared.intent_execution_bindings(),
                    occupancy: self.intent_execution.occupancy(),
                },
                generation: &active,
                inputs: crate::runtime::intent::UiIntentInputOwners {
                    mounted: &self.mounted,
                    application_facts: &self.intent_application_facts,
                    expressions: &self.expressions,
                },
            },
        );
    }

    /// The work the expression owner has done since this session activated.
    /// A generation change never resets it: re-stamping a record counts only
    /// its operand probes, and a rebuild counts as any settle does. A
    /// succession's work counts when it is prepared.
    pub fn expression_work_counters(&self) -> crate::facade::expression::UiExpressionWorkCounters {
        self.expressions.counters()
    }
}
