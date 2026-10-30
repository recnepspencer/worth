use crate::{
    UiEvidenceBudget, UiEvidenceRichness, UiInspectionQuery, UiInspectionRelevance,
    UiInspectionRelevanceOutcome, UiInspectionScope, UiInspectionSupportReport,
    UiInspectionSupportStatus, UiInspectionTarget, UiInspectionTargetClass, UiRelevanceFamily,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UiInspectionRelevanceAdmission {
    target: UiInspectionTarget,
    scope: UiInspectionScope,
    richness: UiEvidenceRichness,
    budget: UiEvidenceBudget,
    relevance: UiInspectionRelevance,
    outcome: UiInspectionRelevanceOutcome,
}

impl UiInspectionRelevanceAdmission {
    pub(crate) fn from_query(query: &UiInspectionQuery) -> Self {
        let outcome = admit_outcome(query);
        Self {
            target: query.target().clone(),
            scope: query.scope(),
            richness: query.richness(),
            budget: query.budget(),
            relevance: query.relevance(),
            outcome,
        }
    }

    pub fn target(&self) -> &UiInspectionTarget {
        &self.target
    }

    pub fn scope(&self) -> UiInspectionScope {
        self.scope
    }

    pub fn richness(&self) -> UiEvidenceRichness {
        self.richness
    }

    pub fn budget(&self) -> UiEvidenceBudget {
        self.budget
    }

    pub fn relevance(&self) -> UiInspectionRelevance {
        self.relevance
    }

    pub fn outcome(&self) -> UiInspectionRelevanceOutcome {
        self.outcome
    }

    pub fn refined_for_support_report(mut self, support_report: UiInspectionSupportReport) -> Self {
        if support_report.status() == UiInspectionSupportStatus::Unsupported
            && matches!(
                self.outcome,
                UiInspectionRelevanceOutcome::Matched | UiInspectionRelevanceOutcome::EmptyLocal
            )
        {
            self.outcome = UiInspectionRelevanceOutcome::UnsupportedScope { scope: self.scope };
        }

        self
    }
}

fn admit_outcome(query: &UiInspectionQuery) -> UiInspectionRelevanceOutcome {
    let filter = query.relevance().filter();
    let aspect_target = matches!(
        query.target(),
        UiInspectionTarget::PublishedAspect { .. } | UiInspectionTarget::ConsumedAspect { .. }
    );

    if filter.family_filter().is_none() && filter.widens_beyond_local() {
        return UiInspectionRelevanceOutcome::ContradictoryRequest;
    }

    if aspect_target && filter.widens_beyond_local() {
        return UiInspectionRelevanceOutcome::NotApplicableToTarget {
            target: UiInspectionTargetClass::from_target(query.target()),
        };
    }

    if filter.widens_beyond_local() && query.budget() == UiEvidenceBudget::Narrow {
        return UiInspectionRelevanceOutcome::BudgetExceeded {
            budget: query.budget(),
        };
    }

    if let Some(family) = filter.family_filter()
        && !target_supports_family(query.target(), family)
    {
        return UiInspectionRelevanceOutcome::NotApplicableToTarget {
            target: UiInspectionTargetClass::from_target(query.target()),
        };
    }

    if matches!(query.target(), UiInspectionTarget::GraphNodeIdentity { .. })
        && filter.family_filter() == Some(UiRelevanceFamily::Obligation)
        && query.relevance().obligation_detail().is_some()
    {
        return UiInspectionRelevanceOutcome::NotApplicableToTarget {
            target: UiInspectionTargetClass::from_target(query.target()),
        };
    }

    if query.relevance().aspect_detail().is_some() && !aspect_target {
        return UiInspectionRelevanceOutcome::NotApplicableToTarget {
            target: UiInspectionTargetClass::from_target(query.target()),
        };
    }

    if query.relevance().aspect_detail().is_some()
        && !matches!(
            filter.family_filter(),
            None | Some(UiRelevanceFamily::Aspect)
        )
    {
        return UiInspectionRelevanceOutcome::NotApplicableToTarget {
            target: UiInspectionTargetClass::from_target(query.target()),
        };
    }

    match (query.target(), query.scope(), filter.family_filter()) {
        (UiInspectionTarget::ProductRoot, UiInspectionScope::Graph, None) => {
            UiInspectionRelevanceOutcome::EmptyLocal
        }
        _ => UiInspectionRelevanceOutcome::Matched,
    }
}

fn target_supports_family(target: &UiInspectionTarget, family: UiRelevanceFamily) -> bool {
    match target {
        UiInspectionTarget::ProductRoot => matches!(
            family,
            UiRelevanceFamily::Declaration
                | UiRelevanceFamily::Admission
                | UiRelevanceFamily::Planning
        ),
        UiInspectionTarget::GraphNodeIdentity { .. } => matches!(
            family,
            UiRelevanceFamily::Graph
                | UiRelevanceFamily::Declaration
                | UiRelevanceFamily::Admission
                | UiRelevanceFamily::Obligation
                | UiRelevanceFamily::Planning
        ),
        UiInspectionTarget::PublishedAspect { .. } | UiInspectionTarget::ConsumedAspect { .. } => {
            matches!(family, UiRelevanceFamily::Aspect)
        }
        UiInspectionTarget::DeclaredSurface { .. }
        | UiInspectionTarget::DeclarationIdentity { .. }
        | UiInspectionTarget::AuthoredSourceProvenance { .. } => matches!(
            family,
            UiRelevanceFamily::Declaration | UiRelevanceFamily::Admission
        ),
        UiInspectionTarget::ObligationGraphNode { .. }
        | UiInspectionTarget::ObligationTouch { .. }
        | UiInspectionTarget::ObligationEvidenceHandle { .. } => {
            matches!(family, UiRelevanceFamily::Obligation)
        }
    }
}
