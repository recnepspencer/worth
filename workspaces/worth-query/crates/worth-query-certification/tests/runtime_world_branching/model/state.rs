use std::collections::BTreeMap;

use super::action::{BranchName, EffectPosture};

const INITIAL_INPUT: &str = "payload";
const INITIAL_DEFINITION_COUNT: usize = 1;

pub(super) struct ExpectedProduct {
    input: String,
    installed_definitions: usize,
}

impl ExpectedProduct {
    pub(super) fn input(&self) -> &str {
        &self.input
    }
}

pub(super) struct IndependentModel {
    products: BTreeMap<BranchName, ExpectedProduct>,
}

impl IndependentModel {
    pub(super) fn new() -> Self {
        Self {
            products: BTreeMap::from([(
                BranchName::Root,
                ExpectedProduct {
                    input: INITIAL_INPUT.to_owned(),
                    installed_definitions: INITIAL_DEFINITION_COUNT,
                },
            )]),
        }
    }

    pub(super) fn fork_from_root(&mut self, branch: BranchName) {
        let input = self.products[&BranchName::Root].input.clone();
        assert!(self
            .products
            .insert(
                branch,
                ExpectedProduct {
                    input,
                    installed_definitions: 0,
                },
            )
            .is_none());
    }

    pub(super) fn apply_effect(&mut self, branch: BranchName, posture: EffectPosture, input: &str) {
        let product = self
            .products
            .get_mut(&branch)
            .expect("the action model must contain its effect target");
        if matches!(posture, EffectPosture::Relational | EffectPosture::Combined) {
            product.input = input.to_owned();
        }
        product.installed_definitions += posture.installed_definition_count();
    }

    pub(super) fn retire(&mut self, branch: BranchName) {
        assert!(self.products.remove(&branch).is_some());
    }

    pub(super) fn product(&self, branch: BranchName) -> &ExpectedProduct {
        &self.products[&branch]
    }

    pub(super) fn products(&self) -> impl Iterator<Item = (BranchName, &ExpectedProduct)> + '_ {
        self.products
            .iter()
            .map(|(branch, expected)| (*branch, expected))
    }

    pub(super) fn installed_definition_count(&self) -> usize {
        self.products
            .values()
            .map(|product| product.installed_definitions)
            .sum()
    }
}
