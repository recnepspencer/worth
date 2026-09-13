use crate::basis_lifecycle::BasisOperationLane;
use crate::domain_installation::{
    WorthQueryBoundCommitPosture, WorthQueryBoundDomainOperation, WorthQueryGraphProviderCallKind,
    WorthQueryOperationGraphAccess, WorthQueryOperationGraphParticipation,
};
use worth_query_execution::facade::runtime::WorthQueryRunningWorkflowRun;

use super::{
    WorthQueryBoundGraphExecutionReceipt, WorthQueryWorkflowAdvanceDenial,
    WorthQueryWorkflowAdvanceDenialKind, WorthQueryWorkflowRunCounters,
};

type BoundGraphParticipation =
    crate::domain_installation::operating_world::WorthQueryBoundGraphParticipation;
type InstalledGraphCommitAuthority =
    crate::domain_installation::graph_participation::WorthQueryInstalledGraphCommitAuthority;

struct StageGraphInvocationPlan<'a> {
    reads: Vec<(&'a BoundGraphParticipation, WorthQueryGraphProviderCallKind)>,
    touches: Vec<&'a BoundGraphParticipation>,
    commit_groups: Vec<(std::sync::Arc<InstalledGraphCommitAuthority>, Vec<String>)>,
}

pub(super) fn invoke_stage_graphs<D, O, F, L: BasisOperationLane>(
    bound: &WorthQueryBoundDomainOperation<D, O, F, L>,
    running: WorthQueryRunningWorkflowRun,
    run_identity: &str,
    stage: &worth_query_installation::facade::WorthQueryPortableWorkflowStage,
    counters: &mut WorthQueryWorkflowRunCounters,
) -> Result<
    (
        WorthQueryRunningWorkflowRun,
        Vec<WorthQueryBoundGraphExecutionReceipt>,
    ),
    WorthQueryWorkflowAdvanceDenial,
> {
    StageGraphInvocation::new(bound, running, run_identity, stage, counters)
        .execute(plan_stage_graph_invocations(bound, stage))
}

fn plan_stage_graph_invocations<'a, D, O, F, L: BasisOperationLane>(
    bound: &'a WorthQueryBoundDomainOperation<D, O, F, L>,
    stage: &worth_query_installation::facade::WorthQueryPortableWorkflowStage,
) -> StageGraphInvocationPlan<'a> {
    let mut reads = Vec::new();
    let mut touches = Vec::new();
    let mut commit_groups: Vec<(std::sync::Arc<InstalledGraphCommitAuthority>, Vec<String>)> =
        Vec::new();
    for participation in bound.graph_participations() {
        if let Some(read) = bound
            .definition()
            .semantics()
            .graph_reads
            .domain_roles()
            .iter()
            .find(|read| {
                read.role == participation.role
                    && stage.semantics().graph_read_roles.contains(&read.role)
                    && matches!(
                        read.participation,
                        WorthQueryOperationGraphParticipation::SeparateAuthority { .. }
                    )
            })
        {
            let kind = match read.access {
                WorthQueryOperationGraphAccess::Observe => WorthQueryGraphProviderCallKind::Observe,
                WorthQueryOperationGraphAccess::Project => WorthQueryGraphProviderCallKind::Project,
            };
            reads.push((participation, kind));
        }
        if stage.semantics().touch_roles.contains(&participation.role) {
            touches.push(participation);
            if bound.commit_posture() == WorthQueryBoundCommitPosture::Atomic {
                if let Some(authority) = &participation.record.commit_authority {
                    match commit_groups.iter_mut().find(|(candidate, _)| {
                        std::sync::Arc::ptr_eq(candidate, authority)
                            && candidate.identity() == authority.identity()
                    }) {
                        Some((_, roles)) => roles.push(participation.role.clone()),
                        None => commit_groups.push((
                            std::sync::Arc::clone(authority),
                            vec![participation.role.clone()],
                        )),
                    }
                }
            }
        }
    }
    StageGraphInvocationPlan {
        reads,
        touches,
        commit_groups,
    }
}

struct StageGraphInvocation<'a, D, O, F, L: BasisOperationLane> {
    bound: &'a WorthQueryBoundDomainOperation<D, O, F, L>,
    running: Option<WorthQueryRunningWorkflowRun>,
    stage_identity: &'a str,
    scope_identity: String,
    counters: &'a mut WorthQueryWorkflowRunCounters,
    receipts: Vec<WorthQueryBoundGraphExecutionReceipt>,
}

impl<'a, D, O, F, L: BasisOperationLane> StageGraphInvocation<'a, D, O, F, L> {
    fn new(
        bound: &'a WorthQueryBoundDomainOperation<D, O, F, L>,
        running: WorthQueryRunningWorkflowRun,
        run_identity: &str,
        stage: &'a worth_query_installation::facade::WorthQueryPortableWorkflowStage,
        counters: &'a mut WorthQueryWorkflowRunCounters,
    ) -> Self {
        Self {
            bound,
            running: Some(running),
            stage_identity: stage.identity(),
            scope_identity: format!("workflow:{run_identity}:stage:{}", stage.identity()),
            counters,
            receipts: Vec::new(),
        }
    }

    fn execute(
        mut self,
        plan: StageGraphInvocationPlan<'a>,
    ) -> Result<
        (
            WorthQueryRunningWorkflowRun,
            Vec<WorthQueryBoundGraphExecutionReceipt>,
        ),
        WorthQueryWorkflowAdvanceDenial,
    > {
        for (participation, kind) in plan.reads {
            self.counters.graph_read_contacts += 1;
            self.contact(participation, kind)?;
        }
        for (authority, mut roles) in plan.commit_groups {
            roles.sort();
            self.counters.commit_admission_contacts += 1;
            let contact = super::commit_execution::contact_workflow_commit_provider(
                &self.scope_identity,
                self.stage_identity,
                &authority,
                &self
                    .bound
                    .graph_participations()
                    .iter()
                    .filter(|participation| roles.contains(&participation.role))
                    .map(|participation| participation.record.installation_authority.as_ref())
                    .collect::<Vec<_>>(),
                self.running
                    .as_ref()
                    .expect("managed workflow run remains live"),
            );
            let receipt = match contact {
                Ok(receipt) => receipt,
                Err(failure) => {
                    let running = self
                        .running
                        .take()
                        .expect("managed workflow run remains live");
                    return Err(self
                        .denial(failure.detail())
                        .with_managed_cleanup(running.abandon().cleanup()));
                }
            };
            self.receipts.push(receipt);
        }
        for participation in plan.touches {
            self.counters.touch_effect_contacts += 1;
            self.contact(participation, WorthQueryGraphProviderCallKind::TouchEffect)?;
        }
        Ok((
            self.running
                .take()
                .expect("managed workflow run remains live"),
            self.receipts,
        ))
    }

    fn contact(
        &mut self,
        participation: &BoundGraphParticipation,
        kind: WorthQueryGraphProviderCallKind,
    ) -> Result<(), WorthQueryWorkflowAdvanceDenial> {
        let running = self
            .running
            .take()
            .expect("managed workflow run remains live");
        match super::managed_graph_progression::execute_workflow_graph(
            running,
            self.stage_identity,
            participation.record.installation_authority.as_ref(),
            kind,
            &self.scope_identity,
        ) {
            Ok((running, receipt)) => {
                self.running = Some(running);
                self.receipts.push(receipt);
                Ok(())
            }
            Err((detail, terminal)) => {
                Err(self.denial(detail).with_managed_cleanup(terminal.cleanup()))
            }
        }
    }

    fn denial(&self, detail: impl Into<String>) -> WorthQueryWorkflowAdvanceDenial {
        WorthQueryWorkflowAdvanceDenial::new(
            WorthQueryWorkflowAdvanceDenialKind::GraphProvider(detail.into()),
            *self.counters,
        )
        .with_graph_receipts(self.receipts.clone())
    }
}
