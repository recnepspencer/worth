//! A declaration supplies producers; canonical Native facts decide the trigger.
use super::*;
use worth_query_execution_lineage::facade::ports::PrimaryGraphIntegrationHandleInternalPort as _;
use worth_query_execution_runtime::facade::ports::ProductPublicationBindingInternalPort as _;
use worth_relational::facade::mvcc::{
    RelationalAuthoredCoupledOwners, RelationalAuthoredCoupledProducerAssembly,
};

pub trait CoupledProducerAssembly: RelationalAuthoredCoupledProducerAssembly {
    fn is_authored_edit(&self) -> bool;
    fn authored_stage_sources(&self) -> Vec<worth_relational::facade::identity::EntityId>;
    fn prepared_members(&self) -> Vec<super::super::CoupledApplicationMember>;
    fn invariant_requirements(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        provider: &WorthQueryPrimaryGraphProvider,
        candidate: &worth_relational::facade::mvcc::ValidatedRelationalProposal,
    ) -> Result<
        Vec<worth_query_installation::facade::WorthQueryInstalledInvariantExecutionRequirement>,
        WorthQueryInvariantExecutionFailure,
    >;
    fn admit_validated(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        provider: &WorthQueryPrimaryGraphProvider,
        receipts: &ClosedCoupledInvariantReceipts<'_>,
    ) -> Result<(), WorthQueryInvariantExecutionFailure>;
}

pub(super) struct NativeAuthoredCoupling<'a> {
    pub(super) owners: RelationalAuthoredCoupledOwners,
    pub(super) assembly: &'a dyn RelationalAuthoredCoupledProducerAssembly,
}
impl WorthQueryPrimaryGraphProvider {
    pub(super) fn native_authored_coupling<'a>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        session: WorthQueryProviderSessionView<'_>,
        material: &mut ApplicationInvariantCandidateMaterial,
        assembly: Option<&'a dyn CoupledProducerAssembly>,
    ) -> Result<Option<NativeAuthoredCoupling<'a>>, WorthQueryInvariantExecutionFailure> {
        let Some(publisher) = &self.graph.application_publication_ref() else {
            return Ok(None);
        };
        let source = assembly.map(|_| {
            let attempts = self.attempts.lock().unwrap_or_else(|p| p.into_inner());
            let staged = attempts.staged_attempt(session).ok_or_else(missing_candidate_failure)?;
            Ok::<_, WorthQueryInvariantExecutionFailure>((
                super::super::application_attempt_state::coupled_publication::authored_retirement_input(self.graph.layout_ref(), &staged)?,
                super::super::application_attempt_state::coupled_publication::authored_source(&staged),
                super::super::authored_retirement_selection::coupling_posture(staged.appendix_attempt()),
            ))
        }).transpose()?;
        if material.restoration_plan.is_none() && source.is_none() {
            return Ok(None);
        }
        let later = assembly
            .map(|value| value.authored_stage_sources())
            .unwrap_or_default();
        let member_edit_source = source.as_ref().and_then(|(retirement, source, posture)| {
            (matches!(posture, super::super::authored_retirement_selection::AuthoredCouplingPosture::LiveOwnerMembers)
                && (retirement.is_some() || material.restoration_plan.is_some())).then_some(*source)
        });
        let owners = self
            .graph
            .with_runtime(execution, |runtime| {
                let basis = material.product.observation().basis().relational_basis();
                let owners = match (&material.restoration_plan, source) {
                    (Some(restoration), _) => runtime.readmit_authored_restoration_stage_owners(
                        execution,
                        &publisher.native,
                        restoration.clone(),
                        !later.is_empty(),
                    )?,
                    (None, Some((selected, source, _))) => Some(match selected {
                        Some((anchor, co_anchors, roles)) => runtime
                            .readmit_authored_coupled_owners(
                                execution,
                                &publisher.native,
                                basis,
                                anchor,
                                &co_anchors,
                                &roles,
                            )?,
                        None => runtime.readmit_edit_coupled_owners(
                            execution,
                            &publisher.native,
                            basis,
                            source,
                        )?,
                    }),
                    (None, None) => unreachable!("an ordinary assembly retains its source"),
                };
                let Some(mut owners) = owners else {
                    return Ok(None);
                };
                if let Some(source) = member_edit_source {
                    owners = runtime.readmit_member_edit_coupled_owners(
                        execution,
                        &publisher.native,
                        owners,
                        source,
                    )?;
                }
                runtime
                    .readmit_later_authored_stage_owners(
                        execution,
                        &publisher.native,
                        owners,
                        &later,
                    )
                    .map(Some)
            })
            .map_err(map_validation_failure)?;
        let Some(owners) = owners.filter(|owners| {
            owners.requires_coupled_assembly()
                || assembly.is_some_and(|assembly| assembly.is_authored_edit())
        }) else {
            // A declaration does not run a producer when Native found no live
            // surviving writer. The established single/cascade path owns this edit.
            return Ok(None);
        };
        let Some(assembly) = assembly else {
            owners
                .require_independent()
                .map_err(map_validation_failure)?;
            unreachable!("nonempty Native owner set requires declared coupling");
        };
        let owners = match material.authored_restoration.take() {
            Some((commit, original, target, co_anchors)) => self
                .graph
                .with_runtime(execution, |runtime| {
                    runtime.bind_authored_coupled_corrective_root(
                        execution,
                        &publisher.native,
                        owners,
                        (commit, (original, target), co_anchors),
                    )
                })
                .map_err(map_validation_failure)?,
            None => owners,
        };
        material.restoration_plan = None;
        material.restoration_publication = None;
        Ok(Some(NativeAuthoredCoupling { owners, assembly }))
    }
}
