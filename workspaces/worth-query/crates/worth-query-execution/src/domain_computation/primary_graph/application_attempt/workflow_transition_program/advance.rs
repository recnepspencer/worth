use super::*;

impl<Schema, Operation, Input, Scope>
    WorthQueryCompleteApplicationReadSet<
        Schema,
        Operation,
        Input,
        Scope,
        WorthQueryProjectedApplicationMutation,
    >
where
    Schema: ApplicationSchema,
    Operation: ApplicationOperationMarkerIdentity<Schema> + 'static,
{
    pub(in crate::domain_computation::primary_graph) fn materialize_workflow_advance<
        Capability,
        Spec,
    >(
        mut self,
        installed: &WorthQueryInstalledApplicationWorkflowSpec<Schema, Spec>,
        instance: super::super::PublishedWorkflowInstanceRef,
        request_kind: WorkflowTransitionRequestKind,
        clock: &crate::domain_computation::runtime_time::WorthQueryRuntimeClock,
    ) -> Result<
        PreparedWorkflowAdvance<Schema, Operation, Input, Scope>,
        WorthQueryApplicationAttemptDenial,
    >
    where
        Capability: ApplicationCapabilityMarkerIdentity<Schema = Schema> + 'static,
        Spec: ApplicationWorkflowSpec<Schema = Schema>,
    {
        if instance.branch() != self.lease.product().product_branch() {
            return Err(denial(
                WorthQueryApplicationAttemptDenialKind::WorkflowTransitionAffinityMismatch,
                self.admission.operation(),
            ));
        }
        if !installed.advance_binding_matches::<Capability, Operation>()
            || self.admission.installed_capability_identity()
                != Some(*installed.advance_capability_identity_bytes())
        {
            return Err(denial(
                WorthQueryApplicationAttemptDenialKind::WorkflowTransitionAuthorityMismatch,
                self.admission.operation(),
            ));
        }
        let layout = self.lease.layout.workflow().clone();
        let published = super::super::PublishedWorkflowDefinitionRef::retained(
            instance.branch(),
            instance.definition_entity_id(),
            instance.definition_content_identity().clone(),
        );
        let (mut compiled, mut facts) = reconstruct_compiled_definition(
            self.lease.handle(),
            self.lease.snapshot(),
            &layout,
            &published,
            installed.program_revision(),
            Spec::IDENTITY.as_str(),
            installed.support_identity_bytes(),
            usize::from(installed.resources().maximum_definition_nodes()),
            usize::from(installed.resources().maximum_definition_connections()),
            WorkflowDefinitionCompilationPosture::Retained,
        )?;
        let subject = self.admission.scope_entity_id();
        let maximum_transitions = usize::try_from(
            installed
                .resources()
                .maximum_retained_transitions_per_instance(),
        )
        .unwrap_or(usize::MAX);
        let mut observed = self.lease.handle().with_runtime(|runtime| {
            super::super::workflow_instance_observation::observe_workflow_instance(
                self.lease.handle(),
                runtime,
                self.lease.snapshot(),
                &layout,
                &instance,
                subject,
                compiled.lineage(),
                &mut compiled,
                maximum_transitions,
                installed.resources().history_reconstruction_budget(),
                super::super::workflow_instance_observation::WorkflowInstanceObservationPurpose::Advance,
            )
        })??;
        let (live_membership, retire_live_membership) = match observed.live_membership {
            Some(membership) => (membership, true),
            None => {
                if request_kind == WorkflowTransitionRequestKind::NavigateBack {
                    return Ok(self.navigation_replay_denial(
                        &layout,
                        instance.entity_id(),
                        std::mem::take(&mut observed.replays),
                        denial(
                            WorthQueryApplicationAttemptDenialKind::WorkflowTransitionAlreadySettled,
                            "settled workflow instance cannot navigate Back",
                        ),
                    ));
                }
                self.lease.handle().with_runtime(|runtime| {
                    observed.ensure_history(
                        self.lease.handle(),
                        runtime,
                        self.lease.snapshot(),
                        &layout,
                        instance.entity_id(),
                        maximum_transitions,
                        &compiled,
                    )
                })??;
                if observed.transitions.is_empty() {
                    return Err(denial(
                        WorthQueryApplicationAttemptDenialKind::WorkflowTransitionAlreadySettled,
                        "settled workflow instance has no exact transition",
                    ));
                }
                let (settled_index, _) = observed
                    .transitions
                    .iter()
                    .enumerate()
                    .max_by_key(|(_, transition)| transition.settlement.occurrence())
                    .expect("settled transition inventory was checked as nonempty");
                let settled = observed.transitions.remove(settled_index);
                let selected = select_terminal_transition(
                    &compiled,
                    instance.entity_id(),
                    &observed.progress_basis,
                )?;
                if !selected.matches_settlement(settled.settlement) {
                    return Err(denial(
                        WorthQueryApplicationAttemptDenialKind::WorkflowTransitionAffinityMismatch,
                        "settled workflow transition does not close the compiled terminal head",
                    ));
                }
                let replay_probe_identity = *selected.identity_bytes();
                // A settled instance holds no live membership, so the check passes.
                let allowance = observed.ensure_step_left()?;
                let (membership, mut settlement_facts) =
                    self.lease.handle().with_runtime(|runtime| {
                        super::super::workflow_instance_observation::recover_settled_live_membership(
                            runtime,
                            self.lease.snapshot(),
                            &layout,
                            settled.entity,
                            selected.identity(),
                            selected.occurrence(),
                        )
                    })??;
                observed.facts.append(&mut settlement_facts);
                facts.append(&mut observed.facts);
                let replays = publication::PreparedWorkflowTransitionReplays::retained(
                    std::mem::take(&mut observed.replays),
                );
                return self
                    .materialize_terminal_transition(
                        &layout, compiled, instance, selected, membership, false, facts, allowance,
                    )
                    .map(|prepared| {
                        prepared.with_replays(replays.with_probe_identity(replay_probe_identity))
                    });
            }
        };
        // A step recorded before the budget is spent or the deadline passes
        // still replays after it.
        let allowance = match observed.ensure_step_left().and_then(|allowance| {
            self.bind_workflow_deadline(clock, &layout, instance.entity_id())
                .map(|_| allowance)
        }) {
            Ok(allowance) => allowance,
            Err(refused) => {
                let entity = instance.entity_id();
                let replays = std::mem::take(&mut observed.replays);
                return Ok(match request_kind {
                    WorkflowTransitionRequestKind::NavigateBack => {
                        self.navigation_replay_denial(&layout, entity, replays, refused)
                    }
                    _ => {
                        let replays =
                            publication::PreparedWorkflowTransitionReplays::retained(replays);
                        self.replay_only_denial(&layout, entity, replays, refused)
                    }
                });
            }
        };
        if request_kind == WorkflowTransitionRequestKind::NavigateBack {
            let selected = match select_navigation_back_transition(
                &compiled,
                instance.entity_id(),
                &observed.progress_basis,
            ) {
                Ok(selected) => selected,
                Err(denial) => {
                    return Ok(self.navigation_replay_denial(
                        &layout,
                        instance.entity_id(),
                        std::mem::take(&mut observed.replays),
                        denial,
                    ));
                }
            };
            let replay_probe_identity = *selected.identity_bytes();
            let replays = publication::PreparedWorkflowTransitionReplays::retained(std::mem::take(
                &mut observed.replays,
            ))
            .for_navigation_back();
            facts.append(&mut observed.facts);
            return self
                .materialize_navigation_back(
                    &layout,
                    compiled,
                    instance,
                    selected,
                    live_membership,
                    facts,
                    allowance,
                )
                .map(|prepared| {
                    prepared.with_replays(replays.with_probe_identity(replay_probe_identity))
                });
        }
        let selection = match request_kind {
            WorkflowTransitionRequestKind::Advance => {
                select_current_transition(&compiled, instance.entity_id(), &observed.progress_basis)
            }
            WorkflowTransitionRequestKind::CollectAssessment { ref node_path } => {
                select_assessment_collection(
                    &compiled,
                    instance.entity_id(),
                    &observed.progress_basis,
                    node_path,
                )
            }
            WorkflowTransitionRequestKind::NavigateBack => unreachable!("Back returned above"),
        };
        let selected = match selection {
            Ok(selected) => selected,
            Err(denial)
                if denial.kind()
                    == WorthQueryApplicationAttemptDenialKind::WorkflowTransitionNodeUnsupported
                    && matches!(request_kind, WorkflowTransitionRequestKind::Advance) =>
            {
                let replays = publication::PreparedWorkflowTransitionReplays::retained(
                    std::mem::take(&mut observed.replays),
                );
                return Ok(self.replay_only_denial(&layout, instance.entity_id(), replays, denial));
            }
            Err(denial) => return Err(denial),
        };
        let replay_probe_identity = *selected.identity_bytes();
        let replays = publication::PreparedWorkflowTransitionReplays::retained(std::mem::take(
            &mut observed.replays,
        ));
        let handoff_facts = matches!(
            selected.kind(),
            SelectedWorkflowTransitionKind::Operation(_)
        )
        .then(|| observed.facts[..observed.handoff_fact_count].to_vec());
        facts.append(&mut observed.facts);
        let prepared = match selected.kind().clone() {
            SelectedWorkflowTransitionKind::AwaitInbound(inbound) => self
                .materialize_inbound_wait_transition(
                    &layout,
                    compiled,
                    instance,
                    selected,
                    live_membership,
                    facts,
                    inbound,
                    allowance,
                ),
            SelectedWorkflowTransitionKind::Assessment(assessment) => self
                .materialize_assessment_requirement(
                    &layout,
                    &compiled,
                    instance,
                    selected,
                    live_membership,
                    false,
                    facts,
                    assessment,
                    observed.progress_basis.progress(),
                    observed.evidence_allowance(installed.resources().maximum_evidence_bytes()),
                    allowance,
                ),
            SelectedWorkflowTransitionKind::Condition(condition) => self
                .materialize_condition_requirement(
                    &layout,
                    *compiled.program_revision(),
                    instance,
                    selected,
                    live_membership,
                    false,
                    facts,
                    condition,
                    allowance,
                ),
            SelectedWorkflowTransitionKind::Approval(approval) => {
                self.materialize_approval_requirement(&layout, instance, selected, facts, approval)
            }
            SelectedWorkflowTransitionKind::EvidenceJoin(policy) => self.materialize_evidence_join(
                &layout,
                compiled,
                instance,
                selected,
                live_membership,
                facts,
                observed.progress_basis.progress(),
                policy,
                allowance,
            ),
            SelectedWorkflowTransitionKind::Terminal => self.materialize_terminal_transition(
                &layout,
                compiled,
                instance,
                selected,
                live_membership,
                retire_live_membership,
                facts,
                allowance,
            ),
            SelectedWorkflowTransitionKind::Operation(operation) => self
                .materialize_operation_requirement(
                    &layout,
                    &compiled,
                    instance,
                    selected,
                    live_membership,
                    false,
                    facts,
                    handoff_facts.expect("operation handoff facts were selected"),
                    observed.progress_basis.progress(),
                    operation,
                    allowance,
                ),
            SelectedWorkflowTransitionKind::NavigationBack => Err(denial(
                WorthQueryApplicationAttemptDenialKind::WorkflowTransitionNodeUnsupported,
                "navigation requires the explicit Back action",
            )),
        }?;
        Ok(prepared.with_replays(replays.with_probe_identity(replay_probe_identity)))
    }
}
