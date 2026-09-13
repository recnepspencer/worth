use crate::runtime::appearance::{
    UiActiveThemeBinding, UiThemeCapabilityReceipt, UiThemeCapabilityReceiptDenial,
    UiThemeSwitchDenial, UiThemeSwitchOrigin, UiThemeSwitchOriginAdmissionDenial,
    UiThemeSwitchRequest,
};
use crate::runtime::rebind::*;

#[derive(Debug)]
pub enum UiNativeThemeSwitchDenial {
    Recovery(super::WorthUiNativeManagedRebindDenial),
    Admission(super::UiThemeSwitchPreparationDenial),
    Scope(UiAffectedScopeDenial),
    Identity(UiIdentityLifecycleDenial),
    Planning(UiRebindPlanningDenial),
    Preparation(UiRebindPreparationDenial),
    ManagedRebindAlreadyInFlight,
}

impl super::WorthUiNativeApplicationShell {
    pub fn prepare_programmatic_theme_switch(
        &mut self,
        surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
        expected_binding_generation: u64,
        capability: UiThemeCapabilityReceipt,
    ) -> Result<UiThemeSwitchRequest, super::UiProgrammaticThemeSwitchPreparationDenial> {
        if self.pending_managed_rebind.is_some() {
            return Err(
                super::UiProgrammaticThemeSwitchPreparationDenial::ManagedRebindAlreadyInFlight,
            );
        }
        self.session.prepare_programmatic_theme_switch(
            surface,
            expected_binding_generation,
            capability,
        )
    }

    pub fn active_theme_binding(
        &self,
        surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    ) -> Option<&UiActiveThemeBinding> {
        self.session.active_theme_binding(surface)
    }

    pub fn admit_appearance_theme(
        &self,
        surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
        definition: &crate::capability::UiThemeDefinitionIdentity,
    ) -> Result<UiThemeCapabilityReceipt, UiThemeCapabilityReceiptDenial> {
        self.session.admit_appearance_theme(surface, definition)
    }

    pub fn issue_theme_switch_origin_from_publication(
        &self,
        receipt: &UiRebindReceipt,
    ) -> Result<UiThemeSwitchOrigin, UiThemeSwitchOriginAdmissionDenial> {
        self.session
            .issue_theme_switch_origin_from_publication(receipt)
    }

    /// Native ingress joins the ordinary classified rebind and its existing handles.
    pub fn begin_theme_switch(
        &mut self,
        request: UiThemeSwitchRequest,
        policy: UiRebindExecutionPolicy,
        execution: UiRebindExecutionRequest,
    ) -> Result<super::WorthUiNativeManagedRebindProgress, UiNativeThemeSwitchDenial> {
        self.begin_managed_theme_switch(request, policy, execution)
    }

    /// Retain the ordinary rebind completion in the shell's existing owner.
    pub fn begin_managed_theme_switch(
        &mut self,
        request: UiThemeSwitchRequest,
        policy: UiRebindExecutionPolicy,
        execution: UiRebindExecutionRequest,
    ) -> Result<super::WorthUiNativeManagedRebindProgress, UiNativeThemeSwitchDenial> {
        let admitted = {
            let outcome = self.execute_theme_switch(request, policy, execution)?;
            match super::native_managed_rebind::detach_required_surface_reconstruction(outcome) {
                super::native_managed_rebind::RequiredSurfaceReconstruction::NotRequired(
                    outcome,
                ) => Ok(super::native_managed_rebind::normalize_managed_outcome(
                    outcome,
                )),
                super::native_managed_rebind::RequiredSurfaceReconstruction::Required(retry) => {
                    Err(retry)
                }
            }
        };
        let retry = match admitted {
            Ok(normalized) => {
                if let super::native_managed_rebind::ManagedRebindNormalization::Published(
                    receipt,
                ) = &normalized
                {
                    self.settle_native_rebind_reconciliation(receipt);
                }
                return Ok(
                    super::native_managed_rebind::retain_normalized_managed_rebind(
                        &mut self.pending_managed_rebind,
                        normalized,
                    ),
                );
            }
            Err(retry) => retry,
        };
        self.begin_rebind_reconstruction(retry, execution.now_tick())
            .map_err(UiNativeThemeSwitchDenial::Recovery)
    }

    fn execute_theme_switch(
        &mut self,
        request: UiThemeSwitchRequest,
        policy: UiRebindExecutionPolicy,
        execution: UiRebindExecutionRequest,
    ) -> Result<UiRebindOutcome<'_>, UiNativeThemeSwitchDenial> {
        if self.pending_managed_rebind.is_some() {
            return Err(UiNativeThemeSwitchDenial::ManagedRebindAlreadyInFlight);
        }
        let turn = request.origin().turn();
        let classification = match self.session.prepare_theme_switch(request) {
            Ok(classification) => classification,
            Err(super::UiThemeSwitchPreparationDenial::Admission(
                UiThemeSwitchDenial::DuplicateOrigin,
            )) => {
                return Ok(UiRebindOutcome::Duplicate(
                    UiDuplicateObservationReceipt::new(turn),
                ));
            }
            Err(super::UiThemeSwitchPreparationDenial::Admission(
                UiThemeSwitchDenial::SupersededOrigin,
            )) => {
                return Ok(UiRebindOutcome::SupersededBeforeEffects(
                    UiRebindSupersededReceipt::before_effects(
                        UiRebindStoppedPhase::ObservationAdmission,
                    ),
                ));
            }
            Err(denial) => return Err(UiNativeThemeSwitchDenial::Admission(denial)),
        };
        let change = match classification {
            crate::runtime::observation::UiChangeClassificationOutcome::ObservedNoChange(
                receipt,
            ) => {
                return Ok(UiRebindOutcome::ObservedNoChange(receipt));
            }
            crate::runtime::observation::UiChangeClassificationOutcome::Changed(change) => change,
            crate::runtime::observation::UiChangeClassificationOutcome::EvidenceOnly(_) => {
                unreachable!("theme binding admission cannot mint authored source succession")
            }
        };
        let scope = self
            .session
            .resolve_affected_scope(change)
            .map_err(UiNativeThemeSwitchDenial::Scope)?;
        let lifecycle = scope
            .resolve_identity_lifecycle()
            .map_err(UiNativeThemeSwitchDenial::Identity)?;
        let plan = self
            .session
            .compile_rebind_plan(lifecycle, policy)
            .map_err(UiNativeThemeSwitchDenial::Planning)?;
        let reconciliation = self
            .pending_native_surface_reconciliation()
            .into_iter()
            .collect::<Vec<_>>();
        let prepared = self
            .session
            .prepare_rebind_with_reconciliation(plan, execution, &reconciliation)
            .map_err(UiNativeThemeSwitchDenial::Preparation)?;
        Ok(prepared.execute(execution.now_tick()))
    }
}
