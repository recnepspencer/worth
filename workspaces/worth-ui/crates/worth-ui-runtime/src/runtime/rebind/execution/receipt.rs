use super::state::UiRebindReservation;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiRebindDisposition {
    Complete,
}

pub struct UiRebindReceipt {
    plan: crate::runtime::rebind::UiRebindPlan,
    publication: UiRebindPublication,
    disposition: UiRebindDisposition,
    _registration: UiRebindReservation,
}

enum UiRebindPublication {
    Changed {
        application: crate::facade::WorthUiApplicationCutoverReceipt,
        mounted: crate::mounting::UiMountedFramePublicationReceipt,
    },
    EvidenceOnly {
        prior: crate::facade::prepared_application_authority::
            WorthUiPreparedApplicationGenerationIdentity,
        active: crate::facade::prepared_application_authority::
            WorthUiPreparedApplicationGenerationIdentity,
    },
    Content {
        generation: crate::facade::prepared_application_authority::
            WorthUiPreparedApplicationGenerationIdentity,
        mounted: crate::mounting::UiMountedFramePublicationReceipt,
    },
    AuthoredContent {
        prior: crate::facade::prepared_application_authority::
            WorthUiPreparedApplicationGenerationIdentity,
        active: crate::facade::prepared_application_authority::
            WorthUiPreparedApplicationGenerationIdentity,
        mounted: crate::mounting::UiMountedFramePublicationReceipt,
    },
}

impl UiRebindReceipt {
    pub(crate) fn changed(
        plan: crate::runtime::rebind::UiRebindPlan,
        mut registration: UiRebindReservation,
        application: crate::facade::WorthUiApplicationCutoverReceipt,
        mounted: crate::mounting::UiMountedFramePublicationReceipt,
    ) -> Result<Self, super::UiRebindInternalDefectOutcome> {
        let matches_plan = application.prior_generation()
            == plan.basis().classification().predecessor_generation()
            && application.active_generation() == plan.basis().candidate_generation()
            && mounted.generation() == plan.basis().candidate_generation();
        if !matches_plan {
            registration
                .retain_recovery()
                .expect("pre-effect admission reserved recovery capacity");
            return Err(super::UiRebindInternalDefectOutcome::published_mismatch(
                plan,
                registration,
                application,
                mounted,
            ));
        }
        registration
            .retain_receipt()
            .expect("pre-effect admission reserved receipt capacity");
        Ok(Self {
            plan,
            publication: UiRebindPublication::Changed {
                application,
                mounted,
            },
            disposition: UiRebindDisposition::Complete,
            _registration: registration,
        })
    }

    pub(crate) fn evidence_only(
        plan: crate::runtime::rebind::UiRebindPlan,
        mut registration: UiRebindReservation,
        prior: crate::facade::prepared_application_authority::
            WorthUiPreparedApplicationGenerationIdentity,
        active: crate::facade::prepared_application_authority::
            WorthUiPreparedApplicationGenerationIdentity,
    ) -> Result<Self, super::UiRebindInternalDefectOutcome> {
        let matches_plan = &prior == plan.basis().classification().predecessor_generation()
            && &active == plan.basis().candidate_generation();
        if !matches_plan {
            registration
                .retain_recovery()
                .expect("pre-publication admission reserved recovery capacity");
            return Err(super::UiRebindInternalDefectOutcome::evidence_mismatch(
                plan,
                registration,
                prior,
                active,
            ));
        }
        registration
            .retain_receipt()
            .expect("pre-publication admission reserved receipt capacity");
        Ok(Self {
            plan,
            publication: UiRebindPublication::EvidenceOnly { prior, active },
            disposition: UiRebindDisposition::Complete,
            _registration: registration,
        })
    }

    pub(crate) fn content(
        plan: crate::runtime::rebind::UiRebindPlan,
        mut registration: UiRebindReservation,
        generation: crate::facade::prepared_application_authority::
            WorthUiPreparedApplicationGenerationIdentity,
        mounted: crate::mounting::UiMountedFramePublicationReceipt,
    ) -> Result<Self, super::UiRebindInternalDefectOutcome> {
        let matches_plan = &generation == plan.basis().classification().predecessor_generation()
            && &generation == plan.basis().candidate_generation()
            && mounted.generation() == &generation;
        if !matches_plan {
            registration
                .retain_recovery()
                .expect("pre-effect admission reserved recovery capacity");
            return Err(super::UiRebindInternalDefectOutcome::content_mismatch(
                plan,
                registration,
                generation,
                mounted,
            ));
        }
        registration
            .retain_receipt()
            .expect("pre-publication admission reserved receipt capacity");
        Ok(Self {
            plan,
            publication: UiRebindPublication::Content {
                generation,
                mounted,
            },
            disposition: UiRebindDisposition::Complete,
            _registration: registration,
        })
    }

    pub(crate) fn authored_content(
        plan: crate::runtime::rebind::UiRebindPlan,
        mut registration: UiRebindReservation,
        prior: crate::facade::prepared_application_authority::
            WorthUiPreparedApplicationGenerationIdentity,
        active: crate::facade::prepared_application_authority::
            WorthUiPreparedApplicationGenerationIdentity,
        mounted: crate::mounting::UiMountedFramePublicationReceipt,
    ) -> Result<Self, super::UiRebindInternalDefectOutcome> {
        let matches_plan = &prior == plan.basis().classification().predecessor_generation()
            && &active == plan.basis().candidate_generation()
            && mounted.generation() == &prior;
        if !matches_plan {
            registration
                .retain_recovery()
                .expect("pre-effect admission reserved recovery capacity");
            return Err(super::UiRebindInternalDefectOutcome::content_mismatch(
                plan,
                registration,
                prior,
                mounted,
            ));
        }
        registration
            .retain_receipt()
            .expect("pre-publication admission reserved receipt capacity");
        Ok(Self {
            plan,
            publication: UiRebindPublication::AuthoredContent {
                prior,
                active,
                mounted,
            },
            disposition: UiRebindDisposition::Complete,
            _registration: registration,
        })
    }

    pub const fn plan(&self) -> &crate::runtime::rebind::UiRebindPlan {
        &self.plan
    }

    pub fn projection_schema_transitions(
        &self,
    ) -> &[crate::runtime::rebind::UiProjectionSchemaTransition] {
        self.plan.projection_schema_transitions()
    }

    pub fn planned_effects(&self) -> &[crate::runtime::rebind::UiRebindDeclarativeEffect] {
        self.plan.effects().effects()
    }

    pub const fn planned_cost(&self) -> crate::runtime::rebind::UiRebindPlanCost {
        self.plan.cost()
    }

    pub fn realized_bindings(&self) -> &[worth_ui_host_contract::UiSurfaceBindingGeneration] {
        match &self.publication {
            UiRebindPublication::Changed { mounted, .. } => mounted.bindings(),
            UiRebindPublication::Content { mounted, .. }
            | UiRebindPublication::AuthoredContent { mounted, .. } => mounted.bindings(),
            UiRebindPublication::EvidenceOnly { .. } => &[],
        }
    }

    pub fn realized_mount_cost(&self) -> Option<crate::mounting::UiMountCostReport> {
        match &self.publication {
            UiRebindPublication::Changed { mounted, .. } => Some(mounted.cost_report()),
            UiRebindPublication::Content { mounted, .. }
            | UiRebindPublication::AuthoredContent { mounted, .. } => Some(mounted.cost_report()),
            UiRebindPublication::EvidenceOnly { .. } => None,
        }
    }

    pub const fn retains_terminal_decision_record(&self) -> bool {
        true
    }

    pub const fn retains_recovery_authority(&self) -> bool {
        false
    }

    pub const fn disposition(&self) -> UiRebindDisposition {
        self.disposition
    }

    pub fn decision_record(&self) -> worth_ui_inspection::UiRebindDecisionRecord {
        super::inspection_projection::project_rebind_decision(self)
    }

    pub fn decision_index(&self) -> worth_ui_inspection::UiRebindDecisionIndex {
        super::inspection_projection::project_rebind_decision_index(self)
    }

    pub(crate) const fn decision_key(&self) -> u64 {
        self._registration.identity()
    }

    pub(crate) const fn session_identity(
        &self,
    ) -> crate::facade::WorthUiActiveApplicationSessionIdentity {
        self.plan.basis().classification().session()
    }

    pub(crate) const fn inspection_disposition(
        &self,
    ) -> worth_ui_inspection::UiRebindDecisionDisposition {
        match &self.publication {
            UiRebindPublication::Changed { .. } => {
                worth_ui_inspection::UiRebindDecisionDisposition::Changed
            }
            UiRebindPublication::Content { .. } | UiRebindPublication::AuthoredContent { .. } => {
                worth_ui_inspection::UiRebindDecisionDisposition::Changed
            }
            UiRebindPublication::EvidenceOnly { .. } => {
                worth_ui_inspection::UiRebindDecisionDisposition::EvidenceOnly
            }
        }
    }

    pub fn prior_generation(
        &self,
    ) -> &crate::facade::prepared_application_authority::WorthUiPreparedApplicationGenerationIdentity
    {
        match &self.publication {
            UiRebindPublication::Changed { application, .. } => application.prior_generation(),
            UiRebindPublication::Content { generation, .. } => generation,
            UiRebindPublication::AuthoredContent { prior, .. } => prior,
            UiRebindPublication::EvidenceOnly { prior, .. } => prior,
        }
    }

    pub fn active_generation(
        &self,
    ) -> &crate::facade::prepared_application_authority::WorthUiPreparedApplicationGenerationIdentity
    {
        match &self.publication {
            UiRebindPublication::Changed { application, .. } => application.active_generation(),
            UiRebindPublication::Content { generation, .. } => generation,
            UiRebindPublication::AuthoredContent { active, .. } => active,
            UiRebindPublication::EvidenceOnly { active, .. } => active,
        }
    }

    pub fn mounted_publication(
        &self,
    ) -> Option<&crate::mounting::UiMountedFramePublicationReceipt> {
        match &self.publication {
            UiRebindPublication::Changed { mounted, .. } => Some(mounted),
            UiRebindPublication::Content { mounted, .. }
            | UiRebindPublication::AuthoredContent { mounted, .. } => Some(mounted),
            UiRebindPublication::EvidenceOnly { .. } => None,
        }
    }

    pub fn application_publication(
        &self,
    ) -> Option<&crate::facade::WorthUiApplicationCutoverReceipt> {
        match &self.publication {
            UiRebindPublication::Changed { application, .. } => Some(application),
            UiRebindPublication::Content { .. }
            | UiRebindPublication::AuthoredContent { .. }
            | UiRebindPublication::EvidenceOnly { .. } => None,
        }
    }

    #[allow(
        clippy::result_large_err,
        reason = "release failure returns the exact affine rebind receipt unchanged"
    )]
    pub fn release_scalar_projection_observation(
        self,
    ) -> Result<worth_ui_query_binding::UiScalarProjectionObservation, Self> {
        if self.plan.scalar_projection_fact_count() != 1 {
            return Err(self);
        }
        let Self {
            plan,
            publication,
            disposition,
            _registration,
        } = self;
        let fact = plan
            .into_scalar_projection_fact()
            .expect("the exact scalar projection count was admitted before release");
        drop((publication, disposition, _registration));
        Ok(fact.into_observation())
    }

    pub fn release_application_scalar_projection_observation(
        self,
    ) -> Result<worth_ui_query_binding::UiApplicationScalarProjectionObservation, Box<Self>> {
        if self.plan.application_scalar_projection_fact_count() != 1 {
            return Err(Box::new(self));
        }
        let Self {
            plan,
            publication,
            disposition,
            _registration,
        } = self;
        let fact = plan
            .into_application_scalar_projection_fact()
            .expect("the exact application scalar count was admitted before release");
        drop((publication, disposition, _registration));
        Ok(fact.into_observation())
    }
}
