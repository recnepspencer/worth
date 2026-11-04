//! Admission seal retained with the actual Home, separately from recorded DATA.
pub(crate) struct HomeTransitionOwner {
    seal: worth_proof::HomeProgramTransitionOwner,
    observation: super::ApplicationHomeTransitionAdmissionObservation,
}

impl HomeTransitionOwner {
    pub(crate) fn new(
        home: worth_foundational::facade::HomeIdentity,
        incarnation: worth_proof::HomeIncarnationFence,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Self, worth_execution::WorkCeilingDenial> {
        let backing = super::admission_backing::HomeTransitionAdmissionBacking::prepare(request)?;
        let observation =
            super::admission_backing::HomeTransitionAdmissionBacking::observation(&backing);
        Ok(Self {
            seal: worth_proof::HomeProgramTransitionOwner::from_retained_owner_backing(
                home.bytes(),
                incarnation,
                backing,
            ),
            observation,
        })
    }

    pub(crate) fn observation(&self) -> super::ApplicationHomeTransitionAdmissionObservation {
        self.observation.clone()
    }

    pub(crate) fn authority(&self) -> worth_proof::HomeProgramTransitionAuthority {
        self.seal.authority()
    }

    pub(crate) fn admit_plan(
        &self,
        commitment: [u8; 32],
    ) -> worth_proof::AdmittedHomeProgramTransitionPlan {
        self.seal.admit_plan(commitment)
    }

    pub(crate) fn admit_step(
        &self,
        commitment: [u8; 32],
    ) -> worth_proof::AdmittedHomeProgramTransitionStep {
        self.seal.admit_step(commitment)
    }
}
