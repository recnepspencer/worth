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

    /// Moves the expression owner to the active generation and to the catalog
    /// the prepared authority installs now. Every writer of the active
    /// generation calls it once the successor owners it reads are in place:
    /// an evidence-only rebind, an application cutover and the initial
    /// mounted establishment. An owner that missed one holds nothing current.
    pub(in crate::facade::entry) fn follow_application_generation(&mut self) {
        let active = self.active_generation_identity();
        self.expressions.follow(
            self.application.prepared_authority().expression_catalog(),
            &crate::runtime::expression::UiExpressionInputs {
                generation: &active,
                mounted: &self.mounted,
                facts: &self.intent_application_facts,
            },
        );
    }

    /// The work the expression owner has done since this session activated.
    /// A generation change never resets it: re-stamping a record counts only
    /// its operand probes, and a rebuild counts as any settle does.
    pub fn expression_work_counters(&self) -> crate::facade::expression::UiExpressionWorkCounters {
        self.expressions.counters()
    }
}
