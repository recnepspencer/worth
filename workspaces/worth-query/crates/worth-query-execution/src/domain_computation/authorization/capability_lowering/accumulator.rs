use worth_runtime_bridge::facade::{
    BridgeAuthorizationClauseContract, BridgeAuthorizationRequirementContract,
    BridgeAuthorizationRuleContract, BridgeAuthorizationRuleEffect,
};

use crate::domain_computation::authorization::capability_registry::{
    WorthQueryCapabilityPathTemplate, WorthQueryCapabilityRuleBinding,
};

pub(in crate::domain_computation::authorization) struct WorthQueryCapabilityRuleSet {
    paths: Vec<WorthQueryCapabilityPathTemplate>,
    rules: Vec<WorthQueryCapabilityRuleBinding>,
}

pub(super) struct WorthQueryCapabilityRuleLoweringAccumulator {
    paths: Vec<WorthQueryCapabilityPathTemplate>,
    rules: Vec<WorthQueryCapabilityRuleBinding>,
}

pub(super) struct WorthQueryAddedCapabilityRule {
    rule_index: usize,
    path_indices: Vec<Vec<usize>>,
}

pub(super) struct WorthQueryCompletedCapabilityRuleLowering {
    paths: Vec<WorthQueryCapabilityPathTemplate>,
    rules: Vec<WorthQueryCapabilityRuleBinding>,
}

pub(super) struct WorthQueryCapabilityRuleLoweringPrefix {
    path_count: usize,
    rule_count: usize,
}

pub(super) struct WorthQueryCompletedCapabilityRulePrefix {
    path_count: usize,
    rules: Vec<WorthQueryCapabilityRuleBinding>,
}

impl WorthQueryCapabilityRuleLoweringAccumulator {
    pub(super) fn new() -> Self {
        Self {
            paths: Vec::new(),
            rules: Vec::new(),
        }
    }

    pub(super) fn path_count(&self) -> usize {
        self.paths.len()
    }

    pub(super) fn add_rule(
        &mut self,
        effect: BridgeAuthorizationRuleEffect,
        requirements: Vec<Vec<WorthQueryCapabilityPathTemplate>>,
    ) -> WorthQueryAddedCapabilityRule {
        let mut indices = Vec::with_capacity(requirements.len());
        for requirement in requirements {
            let mut requirement_indices = Vec::with_capacity(requirement.len());
            for path in requirement {
                requirement_indices.push(self.paths.len());
                self.paths.push(path);
            }
            indices.push(requirement_indices);
        }
        let rule_index = self.rules.len();
        self.rules.push(WorthQueryCapabilityRuleBinding::new(
            bridge_rule(effect, &indices, &self.paths),
            indices.clone(),
        ));
        WorthQueryAddedCapabilityRule {
            rule_index,
            path_indices: indices,
        }
    }

    pub(super) fn add_recorded_actor(
        &mut self,
        requirement: worth_query_declaration::facade::application_capability::ApplicationCapabilityRecordedActorRequirement,
        identity: [u8; 32],
    ) -> usize {
        let index = self.rules.len();
        self.rules
            .push(WorthQueryCapabilityRuleBinding::required_recorded_actor(
                requirement,
                identity,
            ));
        index
    }
    pub(super) fn add_elevation(
        &mut self,
        effect: BridgeAuthorizationRuleEffect,
        path: WorthQueryCapabilityPathTemplate,
    ) -> usize {
        let path_index = self.paths.len();
        let _ = self.add_rule(effect, vec![vec![path]]);
        path_index
    }

    pub(super) fn completed_prefix(&self) -> WorthQueryCapabilityRuleLoweringPrefix {
        WorthQueryCapabilityRuleLoweringPrefix {
            path_count: self.paths.len(),
            rule_count: self.rules.len(),
        }
    }

    pub(super) fn finish(self) -> WorthQueryCompletedCapabilityRuleLowering {
        WorthQueryCompletedCapabilityRuleLowering {
            paths: self.paths,
            rules: self.rules,
        }
    }
}

impl WorthQueryAddedCapabilityRule {
    pub(super) fn rule_index(&self) -> usize {
        self.rule_index
    }

    pub(super) fn into_path_indices(self) -> Vec<Vec<usize>> {
        self.path_indices
    }
}

impl WorthQueryCompletedCapabilityRuleLowering {
    pub(super) fn completed_prefix(
        &self,
        prefix: WorthQueryCapabilityRuleLoweringPrefix,
    ) -> WorthQueryCompletedCapabilityRulePrefix {
        debug_assert!(prefix.path_count <= self.paths.len());
        debug_assert!(prefix.rule_count <= self.rules.len());
        WorthQueryCompletedCapabilityRulePrefix {
            path_count: prefix.path_count,
            rules: self.rules[..prefix.rule_count].to_vec(),
        }
    }

    pub(super) fn rules(&self) -> &[WorthQueryCapabilityRuleBinding] {
        &self.rules
    }

    pub(super) fn into_storage(self) -> WorthQueryCapabilityRuleSet {
        WorthQueryCapabilityRuleSet {
            paths: self.paths,
            rules: self.rules,
        }
    }
}

impl WorthQueryCompletedCapabilityRulePrefix {
    pub(super) fn path_count(&self) -> usize {
        self.path_count
    }

    pub(super) fn rules(&self) -> &[WorthQueryCapabilityRuleBinding] {
        &self.rules
    }

    pub(super) fn into_storage(self) -> Vec<WorthQueryCapabilityRuleBinding> {
        self.rules
    }
}

impl WorthQueryCapabilityRuleSet {
    pub(in crate::domain_computation::authorization) fn paths(
        &self,
    ) -> &[WorthQueryCapabilityPathTemplate] {
        &self.paths
    }

    pub(in crate::domain_computation::authorization) fn rules(
        &self,
    ) -> &[WorthQueryCapabilityRuleBinding] {
        &self.rules
    }
}

fn bridge_rule(
    effect: BridgeAuthorizationRuleEffect,
    requirements: &[Vec<usize>],
    paths: &[WorthQueryCapabilityPathTemplate],
) -> BridgeAuthorizationRuleContract {
    BridgeAuthorizationRuleContract::all(
        effect,
        requirements.iter().map(|indices| {
            BridgeAuthorizationRequirementContract::any(
                indices
                    .iter()
                    .map(|index| BridgeAuthorizationClauseContract::new(paths[*index].identity)),
            )
        }),
    )
}

#[cfg(test)]
mod tests {
    use worth_relational::facade::authorization::RelationalAuthorizationPathPlan;

    use super::*;
    use crate::domain_computation::authorization::capability_registry::WorthQueryCapabilityRequestGuard;

    #[test]
    fn accumulator_atomically_preserves_the_complete_ordered_rule_model() {
        use BridgeAuthorizationRuleEffect::{Prohibited, Required};

        let mut lowering = WorthQueryCapabilityRuleLoweringAccumulator::new();
        let expected = [
            (Required, vec![vec![0]]),
            (Required, vec![vec![1, 2], vec![3]]),
            (Prohibited, vec![vec![4]]),
            (Prohibited, vec![vec![5]]),
            (Prohibited, vec![vec![6]]),
            (Prohibited, vec![vec![7]]),
            (Required, vec![vec![8]]),
            (Required, vec![vec![9]]),
            (Required, vec![vec![10]]),
            (Prohibited, vec![vec![11]]),
            (Prohibited, vec![vec![12]]),
            (Prohibited, vec![vec![13, 14], vec![15]]),
        ];

        let _ = lowering.add_rule(Required, paths(&[&[0]])); // grant
        let _ = lowering.add_rule(Required, paths(&[&[1, 2], &[3]])); // allow
        let _ = lowering.add_rule(Prohibited, paths(&[&[4]])); // deny
        let _ = lowering.add_rule(Prohibited, paths(&[&[5]])); // conflict
        let _ = lowering.add_rule(Prohibited, paths(&[&[6]])); // separation of duty
        let _ = lowering.add_rule(Prohibited, paths(&[&[7]])); // distinct actor
        assert_eq!(lowering.add_elevation(Required, path(8)), 8); // active
        assert_eq!(lowering.add_elevation(Required, path(9)), 9); // not before
        assert_eq!(lowering.add_elevation(Required, path(10)), 10); // not after
        assert_eq!(lowering.add_elevation(Prohibited, path(11)), 11); // expired
        assert_eq!(lowering.add_elevation(Prohibited, path(12)), 12); // self approval
        let approver = lowering.add_rule(Prohibited, paths(&[&[13, 14], &[15]]));
        assert_eq!(approver.into_path_indices(), expected[11].1);

        let storage = lowering.finish().into_storage();
        let paths = storage.paths();
        let rules = storage.rules();
        let identities = paths
            .iter()
            .map(|path| path.identity[0])
            .collect::<Vec<_>>();
        assert_eq!(identities, (0_u8..=15).collect::<Vec<_>>());
        let expected_indices = expected
            .iter()
            .map(|(_, indices)| indices.clone())
            .collect::<Vec<_>>();
        assert_eq!(
            rules
                .iter()
                .map(|rule| rule.path_requirements().to_vec())
                .collect::<Vec<_>>(),
            expected_indices
        );
        assert_eq!(
            rules
                .iter()
                .map(|rule| rule.bridge().effect())
                .collect::<Vec<_>>(),
            expected
                .iter()
                .map(|(effect, _)| *effect)
                .collect::<Vec<_>>()
        );
        for rule in rules {
            let actual = rule
                .bridge()
                .requirements()
                .iter()
                .map(|requirement| {
                    requirement
                        .clauses()
                        .iter()
                        .map(|clause| clause.identity()[0] as usize)
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<_>>();
            assert_eq!(actual, rule.path_requirements());
        }
    }

    fn paths(model: &[&[u8]]) -> Vec<Vec<WorthQueryCapabilityPathTemplate>> {
        model
            .iter()
            .map(|requirement| requirement.iter().copied().map(path).collect())
            .collect()
    }

    fn path(identity: u8) -> WorthQueryCapabilityPathTemplate {
        WorthQueryCapabilityPathTemplate {
            plan: RelationalAuthorizationPathPlan::new([], []),
            identity: [identity; 32],
            guard: WorthQueryCapabilityRequestGuard::Unconditional,
            grant_ordinal: None,
            elevation_ordinals: Vec::new(),
            elevation_resource_ordinal: None,
            context_anchors: Vec::new(),
        }
    }
}
