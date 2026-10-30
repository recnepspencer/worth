use crate::runtime::WorthUiAllocationPlanning;

/// Post-planning allocation result. This is derived candidate truth, never committed truth.
#[derive(Clone, Debug, PartialEq)]
pub struct UiAllocationCandidate {
    planning: WorthUiAllocationPlanning,
    replan_admission:
        Option<crate::runtime::invalidation_narrowing::UiAllocationInvalidationAdmissionContext>,
    resize_basis: Option<crate::runtime::UiResizeAllocationPlanningBasis>,
}

impl UiAllocationCandidate {
    pub(crate) fn from_planning(
        planning: WorthUiAllocationPlanning,
        _: crate::runtime::planning::UiAllocationCandidateMintAuthority,
    ) -> Self {
        Self {
            planning,
            replan_admission: None,
            resize_basis: None,
        }
    }

    pub(crate) fn seal_replan_admission(
        &mut self,
        impact: crate::runtime::WorthUiReplacementImpactClassification,
        narrowing: crate::runtime::WorthUiRuntimeImpactNarrowing,
    ) {
        let admitted_candidate = self.clone();
        let context =
            crate::runtime::invalidation_narrowing::UiAllocationInvalidationAdmissionContext::from_planning_basis(
                self.planning.basis(),
            ).with_allocation_candidate(admitted_candidate)
              .with_replacement_consequences(impact, narrowing);
        self.replan_admission = Some(context);
    }

    pub(crate) fn seal_catalog_successor(&mut self) {
        let admitted_candidate = self.clone();
        self.replan_admission = Some(
            crate::runtime::invalidation_narrowing::UiAllocationInvalidationAdmissionContext::from_planning_basis(
                self.planning.basis(),
            )
            .with_allocation_candidate(admitted_candidate),
        );
    }

    pub(crate) fn seal_replan_successor(
        &mut self,
        impact: std::rc::Rc<crate::runtime::WorthUiReplacementImpactClassification>,
        narrowing: std::rc::Rc<crate::runtime::WorthUiRuntimeImpactNarrowing>,
    ) {
        let admitted_candidate = self.clone();
        let context =
            crate::runtime::invalidation_narrowing::UiAllocationInvalidationAdmissionContext::from_planning_basis(
                self.planning.basis(),
            )
            .with_allocation_candidate(admitted_candidate)
            .with_replacement_lineage(impact, narrowing);
        self.replan_admission = Some(context);
    }

    pub(crate) fn replan_admission(
        &self,
    ) -> &crate::runtime::invalidation_narrowing::UiAllocationInvalidationAdmissionContext {
        self.replan_admission.as_ref().expect(
            "admitted allocation candidates carry graph and replacement-backed replan admission",
        )
    }

    pub(crate) fn replan_admission_opt(
        &self,
    ) -> Option<&crate::runtime::invalidation_narrowing::UiAllocationInvalidationAdmissionContext>
    {
        self.replan_admission.as_ref()
    }

    pub fn is_admitted(&self) -> bool {
        self.planning.is_admitted()
    }

    pub fn measurement_basis(&self) -> &crate::evidence::UiMeasurementBasis {
        self.planning.measurement_basis()
    }

    pub fn planning_identity_digest(&self) -> u64 {
        self.resize_basis
            .as_ref()
            .map_or(self.planning.planning_identity_digest(), |basis| {
                self.planning.planning_identity_digest() ^ basis.identity_digest().rotate_left(17)
            })
    }

    pub fn denial_posture(&self) -> Option<&crate::runtime::WorthUiAllocationPlanningDenial> {
        self.planning.denial_posture()
    }

    pub fn allocation_neighborhood(&self) -> &crate::evidence::UiAllocationNeighborhood {
        self.planning.allocation_neighborhood()
    }

    pub fn allocation_constraint_set(&self) -> Option<&crate::evidence::UiAllocationConstraintSet> {
        self.planning.allocation_constraint_set()
    }

    pub fn truth_category(&self) -> crate::evidence::allocation::UiAllocationTruthCategory {
        crate::evidence::allocation::UiAllocationTruthCategory::Candidate
    }

    pub(crate) fn planning(&self) -> &WorthUiAllocationPlanning {
        &self.planning
    }

    pub(crate) fn seal_resize_basis(
        &mut self,
        basis: crate::runtime::UiResizeAllocationPlanningBasis,
    ) {
        self.resize_basis = Some(basis);
    }
    pub fn resize_basis(&self) -> Option<&crate::runtime::UiResizeAllocationPlanningBasis> {
        self.resize_basis.as_ref()
    }
    pub fn portal_allocation_input(
        &self,
    ) -> Option<&crate::runtime::UiPortalAllocationPlanningBasis> {
        self.planning.basis().portal_allocation_input()
    }

    #[cfg(test)]
    pub(crate) fn basis(&self) -> &crate::runtime::WorthUiAllocationPlanningBasis {
        self.planning.basis()
    }

    #[cfg(test)]
    pub(crate) fn counters(&self) -> crate::runtime::WorthUiAllocationPlanningCounters {
        self.planning.counters()
    }
}
