use std::sync::Arc;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;

use worth_query_installation::facade::WorthQueryInstalledInvariantExecutionRequirement;

use super::{
    WorthQueryAdmittedInvariantStateLoadPlan, WorthQueryInvariantExecutionDenialKind,
    WorthQueryInvariantExecutionFailure, WorthQueryInvariantReceipt,
    WorthQueryInvariantStateLoadAdmission, WorthQueryInvariantStateLoadBinding,
    WorthQueryInvariantStateLoadEvidence, WorthQueryInvariantStateLoadEvidenceView,
    WorthQueryInvariantStateLoadRequestView, WorthQueryInvariantStateLocator,
    WorthQueryInvariantVerdictAdmission,
};
use crate::domain_computation::provider_session::WorthQueryProposedStateInspection;

pub struct WorthQuerySelectedInstalledInvariant<'inspection, 'run> {
    inspection: &'inspection WorthQueryProposedStateInspection<'run>,
    requirement: &'inspection WorthQueryInstalledInvariantExecutionRequirement,
    requirement_identity: Arc<str>,
}

impl WorthQueryProposedStateInspection<'_> {
    pub(in crate::domain_computation) fn provider_session_view(
        &self,
    ) -> crate::domain_computation::provider_session::WorthQueryProviderSessionView<'_> {
        self.proposed.attempt.staged.provider_session_view()
    }

    pub fn installed_invariant_requirements(
        &self,
    ) -> &[WorthQueryInstalledInvariantExecutionRequirement] {
        self.proposed.attempt.staged.plan().invariant_requirements()
    }

    pub fn select_installed_invariant(
        &self,
        slot: &str,
    ) -> Result<WorthQuerySelectedInstalledInvariant<'_, '_>, WorthQueryInvariantExecutionFailure>
    {
        let requirement = self
            .proposed
            .attempt
            .staged
            .plan()
            .invariant_requirements()
            .iter()
            .find(|requirement| requirement.slot() == slot)
            .ok_or_else(|| {
                failure(WorthQueryInvariantExecutionDenialKind::InvariantNotInstalled)
            })?;
        if requirement.executor_role() != self.proposed.attempt.staged.plan().provider_role() {
            return Err(failure(
                WorthQueryInvariantExecutionDenialKind::ExecutorRoleMismatch,
            ));
        }
        Ok(WorthQuerySelectedInstalledInvariant {
            inspection: self,
            requirement,
            requirement_identity: requirement_identity(requirement).into(),
        })
    }
}

impl<'inspection, 'run> WorthQuerySelectedInstalledInvariant<'inspection, 'run> {
    pub fn requirement(&self) -> &WorthQueryInstalledInvariantExecutionRequirement {
        self.requirement
    }

    pub fn admit_state_load_plan(
        self,
        locators: impl IntoIterator<Item = WorthQueryInvariantStateLocator>,
        request: Option<&WorthQueryRequestScope>,
    ) -> Result<
        WorthQueryBoundInvariantExecution<'inspection, 'run>,
        WorthQueryInvariantExecutionFailure,
    > {
        super::request_control::check_live(request)?;
        let max_state_facts = self.requirement.max_state_facts();
        let max_work_units = self.requirement.max_work_units();
        let mut controlled_locators = Vec::new();
        for locator in locators {
            super::request_control::check_live(request)?;
            controlled_locators.push(locator);
        }
        let plan = WorthQueryAdmittedInvariantStateLoadPlan::admit(
            self.inspection.proposed.identity(),
            controlled_locators,
            self.requirement.state_load_families(),
            request,
        )?;
        if max_state_facts.is_some_and(|maximum| plan.locators().len() > maximum) {
            return Err(WorthQueryInvariantExecutionFailure::exhausted(
                WorthQueryInvariantExecutionDenialKind::StateLoadBudgetExceeded,
                "admitted invariant state-load width exhausts the installed budget",
            ));
        }
        let attempt = &self.inspection.proposed.attempt;
        let binding = WorthQueryInvariantStateLoadBinding {
            session_binding_identity: attempt.staged.provisional_binding_identity().into(),
            requirement_identity: self.requirement_identity.clone(),
            proposed_state_identity: self.inspection.proposed.identity().into(),
            attempt_generation: attempt.generation,
            load_plan_identity: plan.identity().into(),
        };
        let admission = WorthQueryInvariantStateLoadAdmission::new(
            binding.clone(),
            &plan,
            max_state_facts,
            max_work_units,
            request.cloned(),
        );
        let invocation = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            attempt.staged.provisional_provider().load_invariant_state(
                attempt.staged.provider_session_view(),
                WorthQueryInvariantStateLoadRequestView::new(&plan),
                admission,
            )
        }));
        super::request_control::check_live(request)?;
        let evidence = match invocation {
            Ok(result) => result?,
            Err(_) => {
                return Err(failure(
                    WorthQueryInvariantExecutionDenialKind::ProviderPanicked,
                ));
            }
        };
        if !evidence.belongs_to(&binding) {
            return Err(failure(
                WorthQueryInvariantExecutionDenialKind::EvidenceSubstitution,
            ));
        }
        Ok(WorthQueryBoundInvariantExecution {
            selected: self,
            plan,
            evidence,
            request: request.cloned(),
            max_work_units,
        })
    }
}

pub struct WorthQueryBoundInvariantExecution<'inspection, 'run> {
    selected: WorthQuerySelectedInstalledInvariant<'inspection, 'run>,
    plan: WorthQueryAdmittedInvariantStateLoadPlan,
    evidence: WorthQueryInvariantStateLoadEvidence,
    request: Option<WorthQueryRequestScope>,
    max_work_units: Option<u64>,
}

#[derive(Clone, Copy)]
pub struct WorthQueryBoundInvariantExecutionView<'a> {
    requirement: &'a WorthQueryInstalledInvariantExecutionRequirement,
    proposed_state_identity: &'a str,
    attempt_generation: u64,
    load_plan: &'a WorthQueryAdmittedInvariantStateLoadPlan,
    load_evidence: WorthQueryInvariantStateLoadEvidenceView<'a>,
}

impl<'a> WorthQueryBoundInvariantExecutionView<'a> {
    pub fn requirement(self) -> &'a WorthQueryInstalledInvariantExecutionRequirement {
        self.requirement
    }

    pub fn proposed_state_identity(self) -> &'a str {
        self.proposed_state_identity
    }

    pub fn attempt_generation(self) -> u64 {
        self.attempt_generation
    }

    pub fn state_load_plan(self) -> &'a WorthQueryAdmittedInvariantStateLoadPlan {
        self.load_plan
    }

    pub fn state_load_evidence(self) -> WorthQueryInvariantStateLoadEvidenceView<'a> {
        self.load_evidence
    }
}

impl WorthQueryBoundInvariantExecution<'_, '_> {
    pub fn execute(
        self,
    ) -> Result<WorthQueryInvariantReceipt, WorthQueryInvariantExecutionFailure> {
        super::request_control::check_live(self.request.as_ref())?;
        let attempt = &self.selected.inspection.proposed.attempt;
        let view = WorthQueryBoundInvariantExecutionView {
            requirement: self.selected.requirement,
            proposed_state_identity: self.selected.inspection.proposed.identity(),
            attempt_generation: attempt.generation,
            load_plan: &self.plan,
            load_evidence: self.evidence.view(),
        };
        let admission = WorthQueryInvariantVerdictAdmission {
            requirement: self.selected.requirement.clone(),
            binding: super::WorthQueryInvariantReceiptBinding {
                requirement_identity: self.selected.requirement_identity,
                provider_identity: attempt.staged.plan().provider_identity().into(),
                provider_generation: attempt.staged.plan().provider_generation(),
                session_binding_identity: attempt.staged.provisional_binding_identity().into(),
                basis_identity: attempt.staged.plan().basis_identity().into(),
                proposed_state_identity: self.selected.inspection.proposed.identity().into(),
                attempt_generation: attempt.generation,
                state_load_plan_identity: self.plan.identity().into(),
                state_load_evidence_identity: self.evidence.identity().into(),
            },
            load_counters: self.evidence.counters(),
            request: self.request.clone(),
            max_work_units: self.max_work_units,
        };
        let expected_binding = admission.binding().clone();
        let invocation = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            attempt.staged.provisional_provider().execute_invariant(
                attempt.staged.provider_session_view(),
                view,
                admission,
            )
        }));
        super::request_control::check_live(self.request.as_ref())?;
        let verdict = match invocation {
            Ok(result) => result?,
            Err(_) => {
                return Err(failure(
                    WorthQueryInvariantExecutionDenialKind::ProviderPanicked,
                ));
            }
        };
        if !verdict.belongs_to(&expected_binding) {
            return Err(failure(
                WorthQueryInvariantExecutionDenialKind::EvidenceSubstitution,
            ));
        }
        Ok(verdict.into_receipt())
    }
}

pub(super) fn requirement_identity(
    requirement: &WorthQueryInstalledInvariantExecutionRequirement,
) -> &str {
    requirement.slot()
}

fn failure(kind: WorthQueryInvariantExecutionDenialKind) -> WorthQueryInvariantExecutionFailure {
    WorthQueryInvariantExecutionFailure::new(kind, "invariant execution progression denied")
}
