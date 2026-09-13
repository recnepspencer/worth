use worth_store_authority::{
    ControlStoreFencingPort, ControlStoreFencingProviderDenial, ControlStoreSelectionCoordinates,
    RecoveryWriteFenceDenial, RecoveryWriteFencePort, RecoveryWriteFenceProviderReceipt,
    RecoveryWriteFenceRecoveryRequest, RecoveryWriteFenceReleaseProviderReceipt,
    RecoveryWriteFenceReleaseRequest, RecoveryWriteFenceRequest, StoreCurrentAuthorityIdentity,
    StoreCurrentAuthorityWitness,
};

pub struct ExactScenarioRecoveryFencePort;

impl RecoveryWriteFencePort for ExactScenarioRecoveryFencePort {
    fn establish(
        &self,
        request: RecoveryWriteFenceRequest,
    ) -> Result<RecoveryWriteFenceProviderReceipt, RecoveryWriteFenceDenial> {
        Ok(RecoveryWriteFenceProviderReceipt::observed(
            [0xf1; 32],
            request.plan_fingerprint(),
            request.expected_current_authority(),
            true,
        ))
    }

    fn release(
        &self,
        request: RecoveryWriteFenceReleaseRequest,
    ) -> Result<RecoveryWriteFenceReleaseProviderReceipt, RecoveryWriteFenceDenial> {
        Ok(RecoveryWriteFenceReleaseProviderReceipt::observed(
            request.fence_identity(),
            request.plan_fingerprint(),
            true,
        ))
    }

    fn recover_active(
        &self,
        request: RecoveryWriteFenceRecoveryRequest,
    ) -> Result<RecoveryWriteFenceProviderReceipt, RecoveryWriteFenceDenial> {
        Ok(RecoveryWriteFenceProviderReceipt::observed(
            request.fence_identity(),
            request.plan_fingerprint(),
            request.expected_current_authority(),
            true,
        ))
    }
}

#[derive(Debug)]
pub struct ExactScenarioControlSelection {
    authority: StoreCurrentAuthorityIdentity,
    coordinates: ControlStoreSelectionCoordinates,
}

impl ExactScenarioControlSelection {
    pub fn current(
        authority: &StoreCurrentAuthorityWitness,
        control: &crate::OperationalControlStore,
    ) -> Self {
        Self {
            authority: authority.authority_identity(),
            coordinates: control
                .observe_selection_coordinates()
                .expect("control selection observation")
                .expect("certification scenario has durable control history"),
        }
    }

    #[cfg(any(test, feature = "certification-test-authority"))]
    pub fn with_selected_prefix_digest_for_controlled_defect(
        mut self,
        prefix_digest: [u8; 32],
    ) -> Self {
        self.coordinates = ControlStoreSelectionCoordinates::new(
            self.coordinates.media_identity_fingerprint(),
            self.coordinates.generation(),
            prefix_digest,
        );
        self
    }
}

impl ControlStoreFencingPort for ExactScenarioControlSelection {
    fn selected_control_store(
        &self,
        current_authority: StoreCurrentAuthorityIdentity,
    ) -> Result<ControlStoreSelectionCoordinates, ControlStoreFencingProviderDenial> {
        if current_authority == self.authority {
            Ok(self.coordinates)
        } else {
            Err(ControlStoreFencingProviderDenial::Unavailable)
        }
    }
}
