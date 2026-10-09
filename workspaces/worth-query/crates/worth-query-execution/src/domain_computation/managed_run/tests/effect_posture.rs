use super::*;

struct EffectProvider {
    partial_effects_may_remain: bool,
    applied_effects: Arc<AtomicUsize>,
    reject_effect: bool,
    panic_after_effect: bool,
}

struct EffectExecution {
    applied_effects: Arc<AtomicUsize>,
    reject_effect: bool,
    panic_after_effect: bool,
}

impl WorthQueryGraphProviderExecution for EffectExecution {
    fn advance(
        &mut self,
        step: &mut WorthQueryGraphProviderStep,
    ) -> Result<WorthQueryGraphProviderStepDisposition, WorthQueryGraphProviderFailure> {
        if self.reject_effect {
            let _ = step.apply_effect(|| -> Result<(), WorthQueryGraphProviderFailure> {
                self.applied_effects.fetch_add(1, Ordering::Relaxed);
                Err(WorthQueryGraphProviderFailure::new(
                    "provider effect rejected",
                ))
            });
            let _ = step.apply_effect(|| {
                self.applied_effects.fetch_add(1, Ordering::Relaxed);
                Ok(())
            });
        } else if self.panic_after_effect {
            step.apply_effect(|| -> Result<(), WorthQueryGraphProviderFailure> {
                self.applied_effects.fetch_add(1, Ordering::Relaxed);
                panic!("provider panicked after applying an effect")
            })?;
        } else {
            step.apply_effect(|| {
                self.applied_effects.fetch_add(1, Ordering::Relaxed);
                Ok(())
            })?;
        }
        WorthQueryGraphProviderStepDisposition::complete("effect-applied")
            .map_err(WorthQueryGraphProviderFailure::new)
    }

    fn dispose(&mut self) -> Result<(), WorthQueryGraphProviderFailure> {
        Ok(())
    }
}

impl WorthQueryGraphParticipationProvider<ManagedGraph> for EffectProvider {
    type Execution = EffectExecution;

    fn execution_resource_support(
        &self,
    ) -> worth_query_admission::facade::resource_admission::WorthQueryExecutionResourceSupport {
        if self.partial_effects_may_remain {
            crate::domain_computation::provider_session::execution_resource_support_with_partial_effects(
                "managed-effect",
                8,
            )
        } else {
            crate::domain_computation::provider_session::execution_resource_support(
                "managed-effect",
                8,
            )
        }
    }

    fn begin(
        &self,
        _call: &WorthQueryGraphProviderCall,
        start: &mut WorthQueryGraphProviderExecutionStart,
    ) -> Result<
        WorthQueryCooperativeGraphProviderExecution<Self::Execution>,
        WorthQueryGraphProviderFailure,
    > {
        admit_provider_execution(
            start,
            EffectExecution {
                applied_effects: Arc::clone(&self.applied_effects),
                reject_effect: self.reject_effect,
                panic_after_effect: self.panic_after_effect,
            },
        )
    }
}

#[test]
fn touch_effect_cannot_cross_an_effect_free_step_contract() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let phase = &active_phase;
        let execution = phase;

        let applied_effects = Arc::new(AtomicUsize::new(0));
        let (running, graph) = managed_graph_effect_run_with_provider(EffectProvider {
            partial_effects_may_remain: false,
            applied_effects: Arc::clone(&applied_effects),
            reject_effect: false,
            panic_after_effect: false,
        });
        let active = start_effect(execution, running, &graph, "effect-free");
        let terminal = match active.advance(execution) {
            WorthQueryDirectGraphStepOutcome::Failed(terminal) => terminal,
            _ => panic!("effect-free bounded step admitted an applied effect"),
        };
        assert_eq!(applied_effects.load(Ordering::Relaxed), 0);
        assert_eq!(terminal.provider_work().attempted_effect_count(), 0);
        assert_eq!(terminal.provider_work().applied_effect_count(), 0);
        assert_eq!(terminal.provider_work().abandoned_call_count(), 1);
        terminal
            .cleanup()
            .expect("denied effect must preserve cleanup authority");
    });
}

#[test]
fn declared_partial_effect_posture_carries_exact_applied_effect_evidence() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let phase = &active_phase;
        let execution = phase;

        let applied_effects = Arc::new(AtomicUsize::new(0));
        let (running, graph) = managed_graph_effect_run_with_provider(EffectProvider {
            partial_effects_may_remain: true,
            applied_effects: Arc::clone(&applied_effects),
            reject_effect: false,
            panic_after_effect: false,
        });
        let completion =
            match start_effect(execution, running, &graph, "partial-effect").advance(execution) {
                WorthQueryDirectGraphStepOutcome::Completed(completion) => completion,
                _ => panic!("declared partial-effect posture did not admit the governed effect"),
            };
        assert_eq!(applied_effects.load(Ordering::Relaxed), 1);
        assert_eq!(completion.receipt().work_report().applied_effect_count(), 1);
        let terminal = completion.into_running().completed().unwrap();
        assert_eq!(terminal.provider_work().attempted_effect_count(), 1);
        assert_eq!(terminal.provider_work().applied_effect_count(), 1);
        terminal
            .cleanup()
            .expect("completed declared effect must retain cleanup authority");
    });
}

#[test]
fn rejected_effect_closure_cannot_claim_an_applied_effect() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let phase = &active_phase;
        let execution = phase;

        let applied_effects = Arc::new(AtomicUsize::new(0));
        let (running, graph) = managed_graph_effect_run_with_provider(EffectProvider {
            partial_effects_may_remain: true,
            applied_effects: Arc::clone(&applied_effects),
            reject_effect: true,
            panic_after_effect: false,
        });
        let terminal =
            match start_effect(execution, running, &graph, "rejected-effect").advance(execution) {
                WorthQueryDirectGraphStepOutcome::Failed(terminal) => terminal,
                _ => panic!("rejected provider effect advanced the managed run"),
            };
        assert_eq!(applied_effects.load(Ordering::Relaxed), 1);
        assert_eq!(terminal.provider_work().attempted_effect_count(), 1);
        assert_eq!(terminal.provider_work().applied_effect_count(), 0);
        assert_eq!(
            terminal.provider_work().session_disposition(),
            WorthQueryManagedProviderSessionDisposition::Uncertain
        );
        let failure = terminal.provider_work().last_step_failure().unwrap();
        assert_eq!(
            failure.invocation(),
            WorthQueryGraphProviderStepInvocationDisposition::Returned
        );
        assert_eq!(failure.invocation_failure_detail(), None);
        assert_eq!(
            failure.latched_provider_failure_detail(),
            Some("provider effect rejected")
        );
        terminal.cleanup().expect("rejected effect should clean up");
    });
}

#[test]
fn panic_after_effect_attempt_preserves_effect_uncertainty() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let phase = &active_phase;
        let execution = phase;

        let applied_effects = Arc::new(AtomicUsize::new(0));
        let (running, graph) = managed_graph_effect_run_with_provider(EffectProvider {
            partial_effects_may_remain: true,
            applied_effects: Arc::clone(&applied_effects),
            reject_effect: false,
            panic_after_effect: true,
        });
        let terminal =
            match start_effect(execution, running, &graph, "panicked-effect").advance(execution) {
                WorthQueryDirectGraphStepOutcome::Failed(terminal) => terminal,
                _ => panic!("panicked provider effect advanced the managed run"),
            };
        assert_eq!(applied_effects.load(Ordering::Relaxed), 1);
        assert_eq!(terminal.provider_work().attempted_effect_count(), 1);
        assert_eq!(terminal.provider_work().applied_effect_count(), 0);
        assert_eq!(
            terminal.provider_work().session_disposition(),
            WorthQueryManagedProviderSessionDisposition::Uncertain
        );
        let failure = terminal.provider_work().last_step_failure().unwrap();
        assert_eq!(
            failure.invocation(),
            WorthQueryGraphProviderStepInvocationDisposition::Panicked
        );
        terminal
            .cleanup()
            .expect("panicked effect should preserve cleanup");
    });
}

fn start_effect(
    execution: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,

    running: WorthQueryRunningDirectRun,
    graph: &WorthQueryInstalledGraphParticipationAuthority,
    scope: &str,
) -> crate::domain_computation::WorthQueryActiveDirectGraphExecution {
    running
        .begin_graph_execution(
            execution,
            graph,
            WorthQueryManagedGraphCallRequest::new(
                WorthQueryGraphProviderCallKind::TouchEffect,
                scope,
            ),
        )
        .expect("installed touch authority should start")
}
