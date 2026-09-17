/// Sole runtime owner for command-route matching, precedence, and prefix
/// occupancy. It owns no command history or inverse-operation authority.
pub(crate) struct UiCommandRoutingRuntimeState {
    persistence: crate::runtime::UiServiceStatePersistencePosture,
    plan: super::plan::UiCommandRoutingPlan,
    prefix: Option<super::prefix::UiCommandPrefixOccupancy>,
    occupancy_revision: u64,
    invocations: u64,
    candidates_visited: u64,
    platform: crate::capability::UiCommandShortcutPlatform,
    policy: crate::declaration::UiCommandRoutingPolicy,
    last_winner: Option<super::UiCommandWonInspectionRecord>,
}

impl UiCommandRoutingRuntimeState {
    pub(crate) fn new(
        persistence: crate::runtime::UiServiceStatePersistencePosture,
        commands: &crate::capability::FrozenCommandCapabilities,
        policy: crate::declaration::UiCommandRoutingPolicy,
    ) -> Self {
        Self {
            persistence,
            plan: super::plan::UiCommandRoutingPlan::compile(commands),
            prefix: None,
            occupancy_revision: 0,
            invocations: 0,
            candidates_visited: 0,
            platform: crate::capability::UiCommandShortcutPlatform::current_target(),
            policy,
            last_winner: None,
        }
    }

    #[cfg(any(test, feature = "certification-support"))]
    pub(crate) fn route_stroke(
        &mut self,
        stroke: crate::capability::UiCommandShortcutStroke,
        repeat: bool,
        context: super::UiCommandRoutingContext,
        application: &crate::runtime::intent::WorthUiActiveApplicationGenerationIdentity,
    ) -> super::UiCommandRoutingOutcome {
        self.route_input_stroke(
            super::input_stroke::UiCommandInputStroke::single(stroke),
            repeat,
            context,
            application,
        )
    }

    pub(crate) fn route_input_stroke(
        &mut self,
        stroke: super::input_stroke::UiCommandInputStroke,
        repeat: bool,
        context: super::UiCommandRoutingContext,
        application: &crate::runtime::intent::WorthUiActiveApplicationGenerationIdentity,
    ) -> super::UiCommandRoutingOutcome {
        let outcome = self.route_input_stroke_unrecorded(stroke, repeat, context, application);
        self.record_outcome(&outcome);
        outcome
    }

    fn route_input_stroke_unrecorded(
        &mut self,
        stroke: super::input_stroke::UiCommandInputStroke,
        repeat: bool,
        context: super::UiCommandRoutingContext,
        application: &crate::runtime::intent::WorthUiActiveApplicationGenerationIdentity,
    ) -> super::UiCommandRoutingOutcome {
        self.invocations = self.invocations.saturating_add(1);
        if let Some(prefix) = self.prefix.take() {
            self.occupancy_revision = self.occupancy_revision.saturating_add(1);
            let first = prefix.first();
            match prefix.currentness(
                application,
                &context,
                self.policy.maximum_prefix_wait_millis(),
            ) {
                super::prefix::UiCommandPrefixCurrentness::Current => {
                    return self.resolve_second(first, stroke, repeat, context, application);
                }
                // A prefix that is no longer current owns nothing. Discarding it
                // must not also consume the stroke the caller just pressed: the
                // stroke is resolved as a fresh first stroke. Only a prefix
                // without a usable time basis stays suppressed, because a fresh
                // resolution could not bound its own occupancy either.
                super::prefix::UiCommandPrefixCurrentness::ContextChanged
                | super::prefix::UiCommandPrefixCurrentness::Expired => {}
                super::prefix::UiCommandPrefixCurrentness::BasisUnavailable => {
                    return super::UiCommandRoutingOutcome::Suppressed(
                        super::UiCommandRoutingSuppression::PrefixBasisUnavailable,
                    );
                }
            }
        }
        self.resolve_first(stroke, repeat, context, application)
    }

    pub(crate) fn cancel_prefix(&mut self) -> bool {
        let cancelled = self.prefix.take().is_some();
        if cancelled {
            self.occupancy_revision = self.occupancy_revision.saturating_add(1);
        }
        cancelled
    }

    pub(crate) fn shutdown(&mut self) -> usize {
        debug_assert_eq!(
            self.persistence,
            crate::runtime::UiServiceStatePersistencePosture::Ephemeral
        );
        self.cancel_prefix();
        let released = self.plan.len();
        self.plan = super::plan::UiCommandRoutingPlan::default();
        self.last_winner = None;
        released
    }

    pub(crate) fn last_winner(&self) -> Option<&super::UiCommandWonInspectionRecord> {
        self.last_winner.as_ref()
    }

    pub(crate) fn resource_counts(&self) -> (usize, usize) {
        (self.plan.len(), usize::from(self.prefix.is_some()))
    }

    #[cfg(any(test, feature = "certification-support"))]
    pub(crate) fn inspect_for_certification(&self) -> (usize, bool, u64, u64) {
        (
            self.plan.len(),
            self.prefix.is_some(),
            self.invocations,
            self.candidates_visited,
        )
    }

    fn resolve_first(
        &mut self,
        stroke: super::input_stroke::UiCommandInputStroke,
        repeat: bool,
        context: super::UiCommandRoutingContext,
        application: &crate::runtime::intent::WorthUiActiveApplicationGenerationIdentity,
    ) -> super::UiCommandRoutingOutcome {
        let candidates = self
            .plan
            .first_stroke_candidates(stroke, self.platform, &context);
        self.candidates_visited = self
            .candidates_visited
            .saturating_add(candidates.len() as u64);
        let (eligible, suppression) =
            super::resolution::eligible_candidates(candidates, repeat, &context, self.policy);
        if eligible.is_empty() {
            return suppression.map_or(
                super::UiCommandRoutingOutcome::Unmatched,
                super::UiCommandRoutingOutcome::Suppressed,
            );
        }
        let (single, prefix): (Vec<_>, Vec<_>) = eligible.into_iter().partition(|candidate| {
            candidate
                .shortcut()
                .is_some_and(|shortcut| shortcut.len() == 1)
        });
        match (single.is_empty(), prefix.is_empty()) {
            (false, true) => super::resolution::resolve_complete(single, application, &context),
            (true, false) => {
                let count = prefix.len();
                drop(prefix);
                self.await_prefix(stroke, count, context, application)
            }
            (false, false) => {
                let single_rank = super::resolution::maximum_rank(&single);
                let prefix_rank = super::resolution::maximum_rank(&prefix);
                match single_rank.cmp(&prefix_rank) {
                    core::cmp::Ordering::Greater => {
                        super::resolution::resolve_complete(single, application, &context)
                    }
                    core::cmp::Ordering::Less => {
                        let count = prefix.len();
                        drop(prefix);
                        self.await_prefix(stroke, count, context, application)
                    }
                    core::cmp::Ordering::Equal => super::UiCommandRoutingOutcome::Suppressed(
                        super::UiCommandRoutingSuppression::PrefixConflict,
                    ),
                }
            }
            (true, true) => super::UiCommandRoutingOutcome::Unmatched,
        }
    }

    fn resolve_second(
        &mut self,
        first: super::input_stroke::UiCommandInputStroke,
        second: super::input_stroke::UiCommandInputStroke,
        repeat: bool,
        context: super::UiCommandRoutingContext,
        application: &crate::runtime::intent::WorthUiActiveApplicationGenerationIdentity,
    ) -> super::UiCommandRoutingOutcome {
        let candidates = self
            .plan
            .first_stroke_candidates(first, self.platform, &context)
            .into_iter()
            .filter(|candidate| {
                let shortcut = candidate
                    .shortcut()
                    .expect("first-stroke index contains only shortcut routes");
                let strokes = shortcut.strokes();
                strokes.len() == 2 && second.matches(strokes[1], self.platform)
            })
            .collect::<Vec<_>>();
        self.candidates_visited = self
            .candidates_visited
            .saturating_add(candidates.len() as u64);
        let (eligible, suppression) =
            super::resolution::eligible_candidates(candidates, repeat, &context, self.policy);
        if eligible.is_empty() {
            return suppression.map_or(
                super::UiCommandRoutingOutcome::Unmatched,
                super::UiCommandRoutingOutcome::Suppressed,
            );
        }
        super::resolution::resolve_complete(eligible, application, &context)
    }

    fn await_prefix(
        &mut self,
        first: super::input_stroke::UiCommandInputStroke,
        candidate_count: usize,
        context: super::UiCommandRoutingContext,
        application: &crate::runtime::intent::WorthUiActiveApplicationGenerationIdentity,
    ) -> super::UiCommandRoutingOutcome {
        let Some(started_at_millis) = super::prefix::monotonic_millis(context.time_basis()) else {
            return super::UiCommandRoutingOutcome::Suppressed(
                super::UiCommandRoutingSuppression::PrefixBasisUnavailable,
            );
        };
        self.occupancy_revision = self.occupancy_revision.saturating_add(1);
        self.prefix = Some(super::prefix::UiCommandPrefixOccupancy::new(
            first,
            self.occupancy_revision,
            application.clone(),
            context,
            started_at_millis,
        ));
        super::UiCommandRoutingOutcome::AwaitingPrefix(super::UiCommandPrefixReceipt::new(
            first.logical(),
            candidate_count,
            self.occupancy_revision,
        ))
    }

    fn record_outcome(&mut self, outcome: &super::UiCommandRoutingOutcome) {
        if let super::UiCommandRoutingOutcome::Routed(receipt) = outcome {
            self.last_winner = Some(super::UiCommandWonInspectionRecord::from_receipt(
                receipt,
                self.invocations,
            ));
        }
    }
}

#[cfg(test)]
impl UiCommandRoutingRuntimeState {
    pub(super) fn with_plan_for_test(plan: super::plan::UiCommandRoutingPlan) -> Self {
        Self::with_plan_and_policy_for_test(
            plan,
            crate::declaration::UiCommandRoutingPolicy::desktop(),
        )
    }

    pub(super) fn with_plan_and_policy_for_test(
        plan: super::plan::UiCommandRoutingPlan,
        policy: crate::declaration::UiCommandRoutingPolicy,
    ) -> Self {
        let platform = plan.platform();
        Self {
            persistence: crate::runtime::UiServiceStatePersistencePosture::Ephemeral,
            plan,
            prefix: None,
            occupancy_revision: 0,
            invocations: 0,
            candidates_visited: 0,
            platform,
            policy,
            last_winner: None,
        }
    }
}
