use super::*;

impl<D: 'static, O, F: 'static, L: BasisOperationLane> WorthQueryPreparedDirectExecution<D, O, F, L>
where
    O: WorthQueryExecutableDomainOperation<
        D,
        F,
        Execution = super::super::super::super::WorthQueryDirectOperation,
    >,
{
    pub(super) fn prepare(
        admitted: WorthQueryAdmittedDirectOperation<D, O, F, L>,
        workspace: &mut crate::runtime::WorthQueryWorkspace,
    ) -> Result<Self, super::super::super::WorthQueryBoundExecutionOutcome<D, O, F, L, O::Output>>
    {
        let mut counters = WorthQueryOperationExecutionCounters {
            runtime_authority_checks: 1,
            ..Default::default()
        };
        let witness =
            crate::domain_installation::WorthQueryInstalledDomainAuthorityWitness::from_authority(
                std::sync::Arc::clone(admitted.bound.operation().domain_authority()),
            );
        if let Err(denial) = workspace.validate_installed_domain_witness::<D>(&witness) {
            return Err(TransitionOutcome::Stale(
                WorthQueryBoundExecutionDenial::new(
                    WorthQueryBoundExecutionDenialKind::RuntimeAuthority(denial.kind()),
                    format!("{denial:?}"),
                    counters,
                ),
            ));
        }
        let resource_evidence = admitted.resource_attempt.evidence().clone();
        let execution_snapshot = workspace.snapshot_identity().map_err(|denial| {
            TransitionOutcome::Denied(WorthQueryBoundExecutionDenial::new(
                WorthQueryBoundExecutionDenialKind::Handle(denial),
                denial.to_string(),
                counters.clone(),
            ))
        })?;
        let conditional = match admitted.evaluate_conditionals(
            workspace,
            &execution_snapshot,
            &resource_evidence,
            &mut counters,
        ) {
            Ok(conditional) => conditional,
            Err(stop) => return Err(conditional_stop_outcome(admitted, counters, stop)),
        };
        let WorthQueryAdmittedDirectOperation {
            bound,
            input,
            executor,
            resource_attempt,
            phase_proof,
        } = admitted;
        let resources = resource_attempt.resources().clone();
        let managed = match workspace.admit_managed_direct_run(
            bound.execution_authority(),
            bound.product(),
            resource_attempt,
        ) {
            Ok(managed) => managed,
            Err(failure) => {
                let detail = failure.detail().to_owned();
                let _ = failure.release();
                return Err(TransitionOutcome::Denied(
                    WorthQueryBoundExecutionDenial::new(
                        WorthQueryBoundExecutionDenialKind::GraphProvider,
                        detail,
                        counters,
                    ),
                ));
            }
        };
        Ok(Self {
            bound,
            input,
            executor,
            phase_proof,
            running: Some(managed.start()),
            resources,
            execution_snapshot,
            conditional,
            resource_evidence,
            counters,
        })
    }
}
