//! Pure semantic model. Runtime identities, comparisons and algorithms never enter here.
use super::CompositeWorldOracle;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug)]
pub struct Occurrence {
    pub parent: Option<usize>,
    pub relational: usize,
    pub signal: usize,
    pub cargo: CompositeWorldOracle,
}
#[derive(Clone, Debug)]
pub struct Branch {
    pub incarnation: usize,
    pub head: usize,
    pub generation: u64,
}
#[derive(Default)]
pub struct ProductModel {
    pub commits: BTreeMap<usize, Occurrence>,
    pub retained: BTreeSet<usize>,
    pub branches: BTreeMap<String, Branch>,
    pub observations: BTreeMap<usize, usize>,
    pub attempts: BTreeMap<usize, (usize, u64, usize)>,
    pub partials: BTreeMap<usize, usize>,
    next_incarnation: usize,
    next_relational: usize,
    next_signal: usize,
}
impl ProductModel {
    pub fn bootstrap() -> Self {
        let mut model = Self {
            next_incarnation: 1,
            next_relational: 1,
            next_signal: 1,
            ..Self::default()
        };
        model.commits.insert(
            0,
            Occurrence {
                parent: None,
                relational: 0,
                signal: 0,
                cargo: CompositeWorldOracle::bootstrap(),
            },
        );
        model.retained.insert(0);
        model.branches.insert(
            "main".into(),
            Branch {
                incarnation: 0,
                head: 0,
                generation: 0,
            },
        );
        model
    }
    pub fn create(&mut self, name: &str, relational: bool, signal: bool) {
        assert!(!self.branches.contains_key(name));
        let source = self.branches["main"].head;
        let head = if relational || signal {
            self.successor(source, relational, signal, None)
        } else {
            source
        };
        self.branches.insert(
            name.into(),
            Branch {
                incarnation: self.next_incarnation,
                head,
                generation: 0,
            },
        );
        self.next_incarnation += 1;
    }
    fn successor(
        &mut self,
        source: usize,
        relational: bool,
        signal: bool,
        amount: Option<u64>,
    ) -> usize {
        let mut next = self.commits[&source].clone();
        next.parent = Some(source);
        if relational {
            next.relational = self.next_relational;
            self.next_relational += 1;
        }
        if signal {
            next.signal = self.next_signal;
            self.next_signal += 1;
        }
        if let Some(amount) = amount {
            next.cargo.change("grain", &amount.to_string());
        }
        let identity = self.commits.len();
        self.commits.insert(identity, next);
        self.retained.insert(identity);
        identity
    }
    pub fn publish(&mut self, name: &str, amount: Option<u64>, signal: bool) {
        assert!(amount.is_some() || signal);
        let source = self.branches[name].head;
        let head = self.successor(source, amount.is_some(), signal, amount);
        let branch = self.branches.get_mut(name).unwrap();
        branch.head = head;
        branch.generation += 1;
    }
    pub fn hold(&mut self, tag: usize, name: &str) {
        assert!(self
            .observations
            .insert(tag, self.branches[name].head)
            .is_none());
    }
    pub fn release(&mut self, tag: usize) {
        assert!(self.observations.remove(&tag).is_some());
    }
    pub fn prepare(&mut self, tag: usize, name: &str) {
        let branch = &self.branches[name];
        assert!(self
            .attempts
            .insert(tag, (branch.incarnation, branch.generation, branch.head))
            .is_none());
    }
    pub fn finish_attempt(&mut self, tag: usize) {
        assert!(self.attempts.remove(&tag).is_some());
    }
    pub fn partial(&mut self, tag: usize, name: &str) {
        self.finish_attempt(tag);
        // Healthy intermediate-effect retention installs an unpublished
        // occurrence. It is never selected by a product branch.
        let successor = self.successor(self.branches[name].head, true, false, Some(5));
        self.partials.insert(tag, successor);
    }
    pub fn cleanup(&mut self, tag: usize) -> usize {
        self.partials.remove(&tag).unwrap()
    }

    pub fn retire(&mut self, name: &str) {
        assert!(self.branches.remove(name).is_some());
    }
    pub fn ancestry(&self, mut head: usize) -> Vec<usize> {
        let mut result = vec![];
        loop {
            result.push(head);
            match self.commits[&head].parent {
                Some(parent) => head = parent,
                None => break,
            }
        }
        result
    }
    pub fn reclaim(&mut self, identity: usize) -> bool {
        let selected = self
            .branches
            .values()
            .map(|b| b.head)
            .chain(self.observations.values().copied())
            .chain(self.partials.values().copied());
        if selected
            .into_iter()
            .any(|head| self.ancestry(head).contains(&identity))
        {
            return false;
        }
        if self
            .retained
            .iter()
            .any(|id| self.commits[id].parent == Some(identity))
        {
            return false;
        }
        self.retained.remove(&identity)
    }
    /// Logical uses, independently counted from live semantic objects. Indices
    /// are head, history, observation, active execution, retained partial.
    pub fn dependencies(&self, relational: bool, basis: usize) -> [usize; 5] {
        let component = |id: usize| {
            if relational {
                self.commits[&id].relational
            } else {
                self.commits[&id].signal
            }
        };
        let heads = self
            .branches
            .values()
            .filter(|b| component(b.head) == basis)
            .count();
        let history = self
            .retained
            .iter()
            .filter(|id| component(**id) == basis)
            .count();
        let claims: BTreeSet<_> = self
            .branches
            .values()
            .map(|b| (b.incarnation, b.generation, b.head))
            .chain(self.attempts.values().copied())
            .collect();
        let observed = claims
            .iter()
            .filter(|(_, _, head)| component(*head) == basis)
            .count();
        let held = self
            .observations
            .values()
            .filter(|id| component(**id) == basis)
            .count();
        let partials = self
            .partials
            .values()
            .filter(|id| component(**id) == basis)
            .count();
        // Prepared attempts reserve capacity. They do not claim active-execution
        // pins until the execution path binds its successor (covered by court parks).
        [heads, history, observed + held, 0, partials]
    }
}
