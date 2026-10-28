//! Structural path meaning shared by each installed path consumer.
use super::{ApplicationQueryRootPathMeaning, ApplicationQueryRootPathTraversal};

impl ApplicationQueryRootPathMeaning {
    pub(crate) fn is_well_formed(&self) -> bool {
        let endpoints_match = match (self.steps().first(), self.steps().last()) {
            (Some(first), Some(last)) => {
                first.parent_entity() == self.start_entity()
                    && last.child_entity() == self.terminal_entity()
            }
            (None, None) => self.start_entity() == self.terminal_entity(),
            _ => unreachable!("one sequence cannot have only one endpoint"),
        };
        endpoints_match
            && self.steps().iter().all(|step| {
                step.traversal() != ApplicationQueryRootPathTraversal::Closure
                    || step.from() == step.to()
            })
            && self
                .steps()
                .windows(2)
                .all(|pair| pair[0].child_entity() == pair[1].parent_entity())
            && self
                .guards()
                .iter()
                .all(|guard| self.entity_after_step(guard.after_step()) == Some(guard.entity()))
    }

    fn entity_after_step(&self, after_step: usize) -> Option<&str> {
        if after_step == 0 {
            Some(self.start_entity())
        } else {
            self.steps()
                .get(after_step - 1)
                .map(super::ApplicationQueryRootPathStep::child_entity)
        }
    }
}
