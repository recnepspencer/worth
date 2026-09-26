use std::collections::{BTreeMap, BTreeSet};

use worth_query_installation::facade::{
    WorthQueryDomainOperationSemanticClosure, WorthQueryOperationEffectContract,
    WorthQueryOperationGraphAccess, WorthQueryOperationInvariantContract,
    WorthQueryOperationTouchContract, WorthQueryOperationTouchScope,
    WorthQueryOperationWorkflowContract,
};

mod aftermath_posture;
#[path = "declared_closure/application.rs"]
mod application;
#[cfg(test)]
pub(super) use aftermath_posture::reversal_posture;
#[cfg(test)]
pub(super) use application::bind_application_candidate_effect_closure;
pub(crate) use application::WorthQueryApplicationEffectPosture;

#[derive(Clone, Debug)]
pub(crate) struct WorthQueryProviderPlanDeclarations {
    direct: BTreeMap<String, WorthQueryProviderPlanDeclaredClosure>,
    workflow: BTreeMap<(String, String), WorthQueryProviderPlanDeclaredClosure>,
    decision_fact_families: Vec<worth_query_installation::facade::WorthQueryDecisionFactFamily>,
    invariant_requirements:
        Vec<worth_query_installation::facade::WorthQueryInstalledInvariantExecutionRequirement>,
    reconciliation_posture: String,
    application_graph_reads:
        Option<worth_query_installation::facade::WorthQueryOperationGraphReadContract>,
    application_touches: Option<worth_query_installation::facade::WorthQueryOperationTouchContract>,
    application_read_touch_overlap:
        Option<worth_query_installation::facade::WorthQueryOperationReadTouchOverlapIndex>,
}

#[derive(Clone, Debug, Default)]
pub(super) struct WorthQueryProviderPlanDeclaredClosure {
    pub(super) read: Vec<String>,
    pub(super) touch: Vec<String>,
    pub(super) effect: Vec<String>,
    pub(super) invariant: Vec<String>,
}

impl Default for WorthQueryProviderPlanDeclarations {
    fn default() -> Self {
        Self {
            direct: BTreeMap::new(),
            workflow: BTreeMap::new(),
            decision_fact_families: Vec::new(),
            invariant_requirements: Vec::new(),
            reconciliation_posture: "not-declared".to_owned(),
            application_graph_reads: None,
            application_touches: None,
            application_read_touch_overlap: None,
        }
    }
}

impl WorthQueryProviderPlanDeclarations {
    pub(crate) fn from_semantics(semantics: &WorthQueryDomainOperationSemanticClosure) -> Self {
        let mut declarations = Self {
            decision_fact_families: semantics.decision_facts.required_families().to_vec(),
            invariant_requirements: semantics.invariant_execution.requirements().to_vec(),
            reconciliation_posture: aftermath_posture::reversal_posture(
                semantics.aftermath.as_ref(),
            ),
            ..Self::default()
        };
        declarations.bind_direct(semantics);
        declarations.bind_workflow(semantics);
        declarations
    }

    pub(super) fn closure(
        &self,
        stage_identity: Option<&str>,
        graph_role: &str,
    ) -> Option<&WorthQueryProviderPlanDeclaredClosure> {
        match stage_identity {
            None => self.direct.get(graph_role),
            Some(stage) => self
                .workflow
                .get(&(stage.to_owned(), graph_role.to_owned())),
        }
    }

    pub(super) fn reconciliation_posture(&self) -> &str {
        &self.reconciliation_posture
    }

    pub(crate) fn decision_fact_families(
        &self,
    ) -> &[worth_query_installation::facade::WorthQueryDecisionFactFamily] {
        &self.decision_fact_families
    }

    pub(super) fn invariant_requirements_for(
        &self,
        stage_identity: Option<&str>,
        graph_role: &str,
    ) -> Vec<worth_query_installation::facade::WorthQueryInstalledInvariantExecutionRequirement>
    {
        let Some(closure) = self.closure(stage_identity, graph_role) else {
            return Vec::new();
        };
        let application_touches = self
            .application_touches
            .as_ref()
            .is_some_and(|touches| !touches.scopes().is_empty());
        if closure.touch.is_empty() && !application_touches {
            return Vec::new();
        }
        self.invariant_requirements
            .iter()
            .filter(|requirement| {
                requirement.executor_role() == graph_role
                    && closure
                        .invariant
                        .iter()
                        .any(|slot| slot == requirement.slot())
            })
            .cloned()
            .collect()
    }

    fn bind_direct(&mut self, semantics: &WorthQueryDomainOperationSemanticClosure) {
        let mut roles = BTreeSet::new();
        for read in semantics.graph_reads.domain_roles() {
            roles.insert(read.role.clone());
            self.direct
                .entry(read.role.clone())
                .or_default()
                .read
                .push(read_binding(&read.role, read.access));
        }
        if let WorthQueryOperationTouchContract::Declared {
            graph_roles,
            scopes,
        } = &semantics.touches
        {
            for role in graph_roles {
                roles.insert(role.clone());
                self.direct.entry(role.clone()).or_default().touch.extend(
                    scopes.iter().filter_map(|scope| match scope {
                        WorthQueryOperationTouchScope::DeclaredDomain(identity) => {
                            Some(identity.as_str().to_owned())
                        }
                        _ => None,
                    }),
                );
            }
        }
        let effects = effect_families(&semantics.effects);
        let invariants = invariant_slots(&semantics.invariants);
        for role in roles {
            let closure = self
                .direct
                .get_mut(&role)
                .expect("collected role has a direct declaration");
            bind_direct_role_closure(closure, &effects, &invariants);
        }
    }

    fn bind_workflow(&mut self, semantics: &WorthQueryDomainOperationSemanticClosure) {
        let WorthQueryOperationWorkflowContract::Declared(workflow) = &semantics.workflow else {
            return;
        };
        for stage in workflow.stages() {
            let stage_semantics = stage.semantics();
            let roles = stage_semantics
                .graph_read_roles
                .iter()
                .chain(&stage_semantics.touch_roles)
                .cloned()
                .collect::<BTreeSet<_>>();
            for role in roles {
                let closure = self
                    .workflow
                    .entry((stage.identity().to_owned(), role.clone()))
                    .or_default();
                if stage_semantics.graph_read_roles.contains(&role) {
                    closure.read.push(role.clone());
                }
                if stage_semantics.touch_roles.contains(&role) {
                    closure.touch.push(role.clone());
                }
                if stage_semantics.touch_roles.contains(&role) {
                    closure.effect = stage_semantics
                        .effect_roles
                        .iter()
                        .map(|effect| effect.as_str().to_owned())
                        .collect();
                }
                closure.invariant = stage_semantics.invariant_roles.clone();
                closure.canonicalize();
            }
        }
    }

    #[cfg(test)]
    pub(crate) fn test_direct(
        graph_role: &str,
        read_access: Option<WorthQueryOperationGraphAccess>,
        touched: bool,
    ) -> Self {
        let mut closure = WorthQueryProviderPlanDeclaredClosure::default();
        if let Some(access) = read_access {
            closure.read.push(read_binding(graph_role, access));
        }
        if touched {
            closure.touch.push(graph_role.to_owned());
            closure.effect.push("mutation".to_owned());
        }
        closure.canonicalize();
        Self {
            direct: [(graph_role.to_owned(), closure)].into_iter().collect(),
            workflow: BTreeMap::new(),
            decision_fact_families: Vec::new(),
            invariant_requirements: Vec::new(),
            reconciliation_posture: if touched {
                "test-declared-compensation"
            } else {
                "not-required"
            }
            .to_owned(),
            application_graph_reads: None,
            application_touches: None,
            application_read_touch_overlap: None,
        }
    }

    #[cfg(test)]
    pub(crate) fn test_direct_with_decision_facts(
        graph_role: &str,
        read_access: WorthQueryOperationGraphAccess,
        decision_fact_families: Vec<worth_query_installation::facade::WorthQueryDecisionFactFamily>,
    ) -> Self {
        let mut declarations = Self::test_direct(graph_role, Some(read_access), false);
        declarations.decision_fact_families = decision_fact_families;
        declarations
    }

    #[cfg(test)]
    pub(crate) fn test_effect_with_decision_facts(
        graph_role: &str,
        decision_fact_families: Vec<worth_query_installation::facade::WorthQueryDecisionFactFamily>,
    ) -> Self {
        let mut declarations = Self::test_direct(graph_role, None, true);
        declarations.decision_fact_families = decision_fact_families;
        declarations
    }

    #[cfg(test)]
    pub(crate) fn test_effect_with_decision_facts_and_invariants(
        graph_role: &str,
        decision_fact_families: Vec<worth_query_installation::facade::WorthQueryDecisionFactFamily>,
        invariant_requirements: Vec<
            worth_query_installation::facade::WorthQueryInstalledInvariantExecutionRequirement,
        >,
    ) -> Self {
        let mut declarations =
            Self::test_effect_with_decision_facts(graph_role, decision_fact_families);
        if let Some(closure) = declarations.direct.get_mut(graph_role) {
            closure.invariant = invariant_requirements
                .iter()
                .map(|requirement| requirement.slot().to_owned())
                .collect();
            closure.canonicalize();
        }
        declarations.invariant_requirements = invariant_requirements;
        declarations
    }

    #[cfg(test)]
    pub(crate) fn test_workflow_stage(
        stage_identity: &str,
        graph_role: &str,
        read_access: WorthQueryOperationGraphAccess,
    ) -> Self {
        let mut closure = WorthQueryProviderPlanDeclaredClosure {
            read: vec![read_binding(graph_role, read_access)],
            ..WorthQueryProviderPlanDeclaredClosure::default()
        };
        closure.canonicalize();
        Self {
            direct: BTreeMap::new(),
            workflow: [((stage_identity.to_owned(), graph_role.to_owned()), closure)]
                .into_iter()
                .collect(),
            decision_fact_families: Vec::new(),
            invariant_requirements: Vec::new(),
            reconciliation_posture: "not-required".to_owned(),
            application_graph_reads: None,
            application_touches: None,
            application_read_touch_overlap: None,
        }
    }

    pub(super) const fn application_graph_reads(
        &self,
    ) -> Option<&worth_query_installation::facade::WorthQueryOperationGraphReadContract> {
        self.application_graph_reads.as_ref()
    }

    pub(super) const fn application_touches(
        &self,
    ) -> Option<&worth_query_installation::facade::WorthQueryOperationTouchContract> {
        self.application_touches.as_ref()
    }

    pub(super) const fn application_read_touch_overlap(
        &self,
    ) -> Option<&worth_query_installation::facade::WorthQueryOperationReadTouchOverlapIndex> {
        self.application_read_touch_overlap.as_ref()
    }
}

impl WorthQueryProviderPlanDeclaredClosure {
    fn canonicalize(&mut self) {
        for values in [
            &mut self.read,
            &mut self.touch,
            &mut self.effect,
            &mut self.invariant,
        ] {
            values.sort();
            values.dedup();
        }
    }
}

pub(super) fn bind_direct_role_closure(
    closure: &mut WorthQueryProviderPlanDeclaredClosure,
    effects: &[String],
    invariants: &[String],
) {
    if !closure.touch.is_empty() {
        closure.effect.clear();
        closure.effect.extend_from_slice(effects);
    }
    closure.invariant.clear();
    closure.invariant.extend_from_slice(invariants);
    closure.canonicalize();
}

fn read_binding(role: &str, access: WorthQueryOperationGraphAccess) -> String {
    match access {
        WorthQueryOperationGraphAccess::Observe => format!("{role}:observe"),
        WorthQueryOperationGraphAccess::Project => format!("{role}:project"),
    }
}

fn effect_families(contract: &WorthQueryOperationEffectContract) -> Vec<String> {
    match contract {
        WorthQueryOperationEffectContract::NotRequired => Vec::new(),
        WorthQueryOperationEffectContract::Declared { effect_families } => effect_families
            .iter()
            .map(|effect| effect.as_str().to_owned())
            .collect(),
    }
}

fn invariant_slots(contract: &WorthQueryOperationInvariantContract) -> Vec<String> {
    match contract {
        WorthQueryOperationInvariantContract::NotRequired => Vec::new(),
        WorthQueryOperationInvariantContract::Declared { invariant_slots } => {
            invariant_slots.clone()
        }
    }
}
