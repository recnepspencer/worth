pub struct CurrentScenarioStagingPort;

impl crate::StagingAuthorizationContinuationPort for CurrentScenarioStagingPort {
    fn observe_revocation(
        &self,
        _request: crate::StagingAuthorizationContinuationRequest,
    ) -> Result<crate::AuthorizationRevocationObservation, crate::AuthorizationProviderFailure>
    {
        Ok(crate::AuthorizationRevocationObservation::NotRevoked { observed_at: 40 })
    }
}

impl crate::workflow::StagedWalApplicationPort for CurrentScenarioStagingPort {
    fn apply_staged_wal(
        &self,
        request: crate::workflow::StagedWalApplicationRequest<'_>,
    ) -> Result<
        crate::workflow::StagedWalApplicationProviderReceipt,
        crate::workflow::StagedWalApplicationDenial,
    > {
        let source = request.replay_source();
        let staging = request.staging();
        Ok(
            crate::workflow::StagedWalApplicationProviderReceipt::observed(
                request.application_identity(),
                staging.staging_plan_fingerprint(),
                source.identity(),
                source.interval(),
                source.frame_count(),
                request.target_frontier_identity(),
                true,
            ),
        )
    }
}
