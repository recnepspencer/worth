use crate::basis_lifecycle::BasisOperationLane;
use crate::domain_installation::{
    WorthQueryBoundCommitPosture, WorthQueryBoundDomainOperation, WorthQueryGraphProviderCallKind,
    WorthQueryOperationGraphAccess, WorthQueryOperationGraphParticipation,
    WorthQueryOperationTouchContract,
};
use worth_query_execution::facade::runtime::WorthQueryRunningDirectRun;

use super::{
    WorthQueryBoundExecutionDenial, WorthQueryBoundExecutionDenialKind,
    WorthQueryBoundGraphExecutionReceipt, WorthQueryOperationExecutionCounters,
};

type BoundGraphParticipation =
    crate::domain_installation::operating_world::WorthQueryBoundGraphParticipation;
type InstalledGraphCommitAuthority =
    crate::domain_installation::graph_participation::WorthQueryInstalledGraphCommitAuthority;

struct BoundGraphInvocationPlan<'a> {
    reads: Vec<(&'a BoundGraphParticipation, WorthQueryGraphProviderCallKind)>,
    touches: Vec<&'a BoundGraphParticipation>,
    commit_groups: Vec<(std::sync::Arc<InstalledGraphCommitAuthority>, Vec<String>)>,
}

pub(super) fn invoke_bound_graphs<D, O, F, L: BasisOperationLane>(
    bound: &WorthQueryBoundDomainOperation<D, O, F, L>,
    running: WorthQueryRunningDirectRun,
    counters: &mut WorthQueryOperationExecutionCounters,
) -> Result<
    (
        WorthQueryRunningDirectRun,
        Vec<WorthQueryBoundGraphExecutionReceipt>,
    ),
    WorthQueryBoundExecutionDenial,
> {
    BoundGraphInvocation::new(bound, running, counters).execute(plan_bound_graph_invocations(bound))
}

fn plan_bound_graph_invocations<D, O, F, L: BasisOperationLane>(
    bound: &WorthQueryBoundDomainOperation<D, O, F, L>,
) -> BoundGraphInvocationPlan<'_> {
    let semantics = bound.definition().semantics();
    let mut reads = Vec::new();
    let mut touches = Vec::new();
    let mut commit_groups: Vec<(std::sync::Arc<InstalledGraphCommitAuthority>, Vec<String>)> =
        Vec::new();
    for participation in bound.graph_participations() {
        if let Some(read) = semantics.graph_reads.domain_roles().iter().find(|read| {
            read.role == participation.role
                && matches!(
                    read.participation,
                    WorthQueryOperationGraphParticipation::SeparateAuthority { .. }
                )
        }) {
            let kind = match read.access {
                WorthQueryOperationGraphAccess::Observe => WorthQueryGraphProviderCallKind::Observe,
                WorthQueryOperationGraphAccess::Project => WorthQueryGraphProviderCallKind::Project,
            };
            reads.push((participation, kind));
        }
        if matches!(&semantics.touches, WorthQueryOperationTouchContract::Declared { graph_roles, .. } if graph_roles.contains(&participation.role))
        {
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
    BoundGraphInvocationPlan {
        reads,
        touches,
        commit_groups,
    }
}

struct BoundGraphInvocation<'a, D, O, F, L: BasisOperationLane> {
    bound: &'a WorthQueryBoundDomainOperation<D, O, F, L>,
    running: Option<WorthQueryRunningDirectRun>,
    scope_identity: String,
    counters: &'a mut WorthQueryOperationExecutionCounters,
    receipts: Vec<WorthQueryBoundGraphExecutionReceipt>,
}

impl<'a, D, O, F, L: BasisOperationLane> BoundGraphInvocation<'a, D, O, F, L> {
    fn new(
        bound: &'a WorthQueryBoundDomainOperation<D, O, F, L>,
        running: WorthQueryRunningDirectRun,
        counters: &'a mut WorthQueryOperationExecutionCounters,
    ) -> Self {
        Self {
            bound,
            running: Some(running),
            scope_identity: format!("direct-capability:{}", bound.capability_identity()),
            counters,
            receipts: Vec::new(),
        }
    }

    fn execute(
        mut self,
        plan: BoundGraphInvocationPlan<'a>,
    ) -> Result<
        (
            WorthQueryRunningDirectRun,
            Vec<WorthQueryBoundGraphExecutionReceipt>,
        ),
        WorthQueryBoundExecutionDenial,
    > {
        self.contact_reads(plan.reads)?;
        self.contact_commit_groups(plan.commit_groups)?;
        self.contact_touches(plan.touches)?;
        Ok((
            self.running
                .take()
                .expect("managed direct run remains live"),
            self.receipts,
        ))
    }

    fn contact_reads(
        &mut self,
        reads: Vec<(&BoundGraphParticipation, WorthQueryGraphProviderCallKind)>,
    ) -> Result<(), WorthQueryBoundExecutionDenial> {
        for (participation, kind) in reads {
            self.contact(participation, kind)?;
        }
        Ok(())
    }

    fn contact_touches(
        &mut self,
        touches: Vec<&BoundGraphParticipation>,
    ) -> Result<(), WorthQueryBoundExecutionDenial> {
        for participation in touches {
            self.contact(participation, WorthQueryGraphProviderCallKind::TouchEffect)?;
        }
        Ok(())
    }

    fn contact(
        &mut self,
        participation: &BoundGraphParticipation,
        kind: WorthQueryGraphProviderCallKind,
    ) -> Result<(), WorthQueryBoundExecutionDenial> {
        self.counters.graph_provider_contacts += 1;
        let running = self
            .running
            .take()
            .expect("managed direct run remains live");
        match super::managed_graph_progression::execute_direct_graph(
            running,
            participation.record.installation_authority.as_ref(),
            kind,
            &self.scope_identity,
        ) {
            Ok((running, receipt)) => {
                self.running = Some(running);
                self.receipts.push(receipt);
                Ok(())
            }
            Err((detail, terminal)) => Err(WorthQueryBoundExecutionDenial::new(
                WorthQueryBoundExecutionDenialKind::GraphProvider,
                detail,
                *self.counters,
            )
            .with_graph_receipts(self.receipts.clone())
            .with_managed_cleanup(terminal.cleanup())),
        }
    }

    fn contact_commit_groups(
        &mut self,
        commit_groups: Vec<(std::sync::Arc<InstalledGraphCommitAuthority>, Vec<String>)>,
    ) -> Result<(), WorthQueryBoundExecutionDenial> {
        for (authority, mut roles) in commit_groups {
            roles.sort();
            self.counters.graph_provider_contacts += 1;
            let running = self
                .running
                .as_ref()
                .expect("managed direct run remains live");
            let contact = super::commit_execution::contact_direct_commit_provider(
                &self.scope_identity,
                &authority,
                &self
                    .bound
                    .graph_participations()
                    .iter()
                    .filter(|participation| roles.contains(&participation.role))
                    .map(|participation| participation.record.installation_authority.as_ref())
                    .collect::<Vec<_>>(),
                running,
            );
            let receipt = match contact {
                Ok(receipt) => receipt,
                Err(failure) => {
                    let running = self
                        .running
                        .take()
                        .expect("managed direct run remains live");
                    return Err(WorthQueryBoundExecutionDenial::new(
                        WorthQueryBoundExecutionDenialKind::GraphProvider,
                        failure.detail(),
                        *self.counters,
                    )
                    .with_graph_receipts(self.receipts.clone())
                    .with_managed_cleanup(running.abandon().cleanup()));
                }
            };
            self.receipts.push(receipt);
        }
        Ok(())
    }
}
