//! Admission seal retained with the actual Home, separately from recorded DATA.
pub(crate) struct HomeTransitionOwner {
    pub(super) admission: super::HomeTransitionAdmission,
    seal: worth_proof::HomeProgramTransitionOwner,
    observation: super::ApplicationHomeTransitionAdmissionObservation,
    publication: std::sync::OnceLock<std::sync::Arc<super::HomeIntentPublication>>,
    resumed_executions:
        std::sync::Mutex<Option<worth_execution::ExecutionArray<std::sync::Mutex<bool>>>>,
}

impl HomeTransitionOwner {
    pub(crate) fn new(
        home: worth_foundational::facade::HomeIdentity,
        incarnation: worth_proof::HomeIncarnationFence,
        admission: super::HomeTransitionAdmission,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Self, worth_execution::WorkCeilingDenial> {
        let backing = super::admission_backing::HomeTransitionAdmissionBacking::prepare(request)?;
        let observation =
            super::admission_backing::HomeTransitionAdmissionBacking::observation(&backing);
        Ok(Self {
            admission,
            seal: worth_proof::HomeProgramTransitionOwner::from_retained_owner_backing(
                home.bytes(),
                incarnation,
                backing,
            ),
            observation,
            publication: std::sync::OnceLock::new(),
            resumed_executions: std::sync::Mutex::new(None),
        })
    }

    pub(super) fn claim_resumed_execution(
        &self,
        step: u32,
        steps: u32,
        commitment: [u8; 32],
        association: [u8; 32],
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<
        worth_proof::AdmittedHomeProgramTransitionExecution,
        super::ApplicationHomeTransitionIntentDenial,
    > {
        use super::ApplicationHomeTransitionIntentDenial as Denial;
        let mut table = self
            .resumed_executions
            .lock()
            .unwrap_or_else(|cause| cause.into_inner());
        if table.is_none() {
            let mut cells =
                worth_execution::ExecutionRequestArrayBuilder::allocate(*request, steps as usize)
                    .map_err(Denial::Backing)?;
            for _ in 0..steps {
                request.consult().map_err(Denial::Execution)?;
                cells
                    .push(std::sync::Mutex::new(false))
                    .map_err(Denial::Backing)?;
            }
            *table = Some(cells.seal().map_err(Denial::Backing)?);
        }
        let table = table.as_ref().expect("same original assignment table");
        if table.elements().len() != steps as usize {
            return Err(Denial::TransitionPlanDrift);
        }
        let cell = table
            .elements()
            .get(step as usize)
            .ok_or(Denial::AssignmentCount)?;
        let mut claimed = cell.lock().unwrap_or_else(|cause| cause.into_inner());
        if *claimed {
            return Err(Denial::StepAlreadyClaimed);
        }
        request.consult().map_err(Denial::Execution)?;
        *claimed = true;
        Ok(self.seal.admit_execution(commitment, association))
    }

    pub(crate) fn reserved_attempt_frontier(&self) -> Option<u64> {
        self.admission.history().reserved_attempt_frontier()
    }

    pub(crate) fn observation(&self) -> super::ApplicationHomeTransitionAdmissionObservation {
        self.observation.clone()
    }
    pub(crate) fn intent(&self) -> Option<&super::HomeProgramTransitionIntent> {
        self.publication().map(|publication| &publication.intent)
    }

    pub(super) fn publication(&self) -> Option<&super::HomeIntentPublication> {
        self.publication.get().map(std::sync::Arc::as_ref)
    }

    pub(super) fn install_publication(
        &self,
        publication: std::sync::Arc<super::HomeIntentPublication>,
    ) -> Result<(), super::ApplicationHomeTransitionIntentDenial> {
        match self.publication.set(publication) {
            Ok(()) => Ok(()),
            Err(publication) => {
                let original = self
                    .publication
                    .get()
                    .expect("a set failure retains the original intent");
                original.intent.require_plan(
                    publication.intent.successor(),
                    publication.intent.plan().digest(),
                    publication.intent.plan().steps(),
                    publication.intent.plan().target_meaning(),
                    publication.intent.bodies(),
                )?;
                if original.intent.plan() != publication.intent.plan()
                    || original.intent.native_predecessor()
                        != publication.intent.native_predecessor()
                    || original.intent.predecessor() != publication.intent.predecessor()
                {
                    return Err(super::ApplicationHomeTransitionIntentDenial::SourceMismatch);
                }
                Ok(())
            }
        }
    }

    pub(crate) fn committed_intent(
        &self,
        current: &worth_foundational::facade::ApplicationHomeBinding,
    ) -> Result<
        Option<&super::HomeProgramTransitionIntent>,
        super::ApplicationHomeTransitionIntentDenial,
    > {
        if let Some(publication) = self.publication() {
            return match publication.observation() {
                Some(super::ApplicationHomeTransitionIntentOutcome::Committed) => {
                    Ok(Some(&publication.intent))
                }
                Some(super::ApplicationHomeTransitionIntentOutcome::Pending) | None => {
                    Err(super::ApplicationHomeTransitionIntentDenial::IntentNotCommitted)
                }
                Some(super::ApplicationHomeTransitionIntentOutcome::Refused(_))
                | Some(super::ApplicationHomeTransitionIntentOutcome::Unresolved)
                | Some(super::ApplicationHomeTransitionIntentOutcome::SourceMismatch) => {
                    Err(super::ApplicationHomeTransitionIntentDenial::IntentNotCommitted)
                }
            };
        }
        self.admission.history().pending(current)
    }

    pub(crate) fn admit_catalog<Source>(
        &self,
        commitment: [u8; 32],
        source: Source,
    ) -> worth_proof::AdmittedHomeProgramTransitionCatalog<Source> {
        self.seal.admit_catalog(commitment, source)
    }

    pub(crate) fn authority(&self) -> worth_proof::HomeProgramTransitionAuthority {
        self.seal.authority()
    }

    pub(crate) fn required(&self) -> Option<worth_proof::HomeProgramTransitionRequired> {
        match &self.admission {
            super::HomeTransitionAdmission::Current { history: _ } => None,
            super::HomeTransitionAdmission::Requested {
                history: _,
                successor,
            } => Some(
                self.seal
                    .require_transition(crate::descriptor::binding::commitment(successor)),
            ),
        }
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
        caller_association: [u8; 32],
        body: [u8; 32],
        source: worth_relational::facade::branch::RelationalBranchBasisAdmissionIdentity,
    ) -> worth_proof::AdmittedHomeProgramTransitionStep<
        worth_relational::facade::branch::RelationalBranchBasisAdmissionIdentity,
    > {
        self.seal
            .admit_step(commitment, caller_association, body, source)
    }
}
