//! Atomic recipe registration and exact selected-registry owner checks.
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use super::definition::RegisteredDerivedRelation;
use super::DerivedRelationNode as Node;
use super::{DerivedRelationDefinitionParts as Parts, DerivedRelationDenial as Denial};
use super::{DerivedRelationHandle as Handle, DerivedRelationIdentity as Identity};
use crate::schema::data::RelationalSchemaRegistry;

impl RelationalSchemaRegistry {
    /// Registers a complete finite named DAG atomically. Existing entries may
    /// be referenced but cannot be replaced, even with equal descriptive parts.
    pub fn register_derived_relations(mut self, additions: Vec<Parts>) -> Result<Self, Denial> {
        let mut definitions = self
            .derived_relations
            .iter()
            .map(|(key, entry)| (key.clone(), entry.parts.clone()))
            .collect::<BTreeMap<_, _>>();
        let mut pending = BTreeSet::new();
        for parts in additions {
            let identity = parts.identity.clone();
            if definitions.insert(identity.clone(), parts).is_some() {
                return Err(Denial::DuplicateDefinition(identity));
            }
            pending.insert(identity);
        }
        let mut dependencies = definitions
            .iter()
            .map(|(identity, parts)| {
                super::validation::expression(parts, &definitions, &self)
                    .map(|sources| (identity.clone(), sources))
            })
            .collect::<Result<BTreeMap<_, _>, _>>()?;
        let mut remaining = BTreeMap::new();
        let mut users: BTreeMap<Identity, Vec<Identity>> = BTreeMap::new();
        let mut ready = BTreeSet::new();
        for identity in &pending {
            let mut count = 0;
            for source in &dependencies[identity].dependencies {
                if pending.contains(source) {
                    count += 1;
                    users
                        .entry(source.clone())
                        .or_default()
                        .push(identity.clone());
                }
            }
            if count == 0 {
                ready.insert(identity.clone());
            }
            remaining.insert(identity.clone(), count);
        }
        while let Some(identity) = ready.pop_first() {
            let parts = definitions
                .remove(&identity)
                .expect("validated pending definition");
            let mut leaves = BTreeSet::new();
            let mut domains = BTreeSet::new();
            for node in &parts.nodes {
                match node {
                    Node::Stored { relation, from, to } => {
                        leaves.insert((*relation, *from, *to));
                    }
                    Node::Derived(source) => {
                        leaves.extend(self.derived_relations[source].leaves.iter().copied());
                        domains.extend(self.derived_relations[source].domains.iter().cloned());
                    }
                    Node::CarrierLift { domain, .. } => {
                        domains.insert(domain.clone());
                    }
                    Node::Compose { .. }
                    | Node::Reverse { .. }
                    | Node::WithoutSelfEdges { .. }
                    | Node::Union { .. }
                    | Node::Intersection { .. } => {}
                }
            }
            let carriers = domains
                .iter()
                .map(|identity| self.projected_member_domains[identity].parts.carrier)
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect();
            self.derived_relations.insert(
                identity.clone(),
                Arc::new(RegisteredDerivedRelation {
                    parts,
                    leaves: leaves.into_iter().collect(),
                    domains: domains.into_iter().collect(),
                    carriers,
                    endpoints: dependencies
                        .remove(&identity)
                        .expect("validated pending expression")
                        .endpoints,
                }),
            );
            pending.remove(&identity);
            for user in users.remove(&identity).unwrap_or_default() {
                let count = remaining.get_mut(&user).expect("pending dependent");
                *count -= 1;
                if *count == 0 {
                    ready.insert(user);
                }
            }
        }
        if !pending.is_empty() {
            return Err(Denial::CyclicDefinitions);
        }
        Ok(self)
    }

    /// Obtain an actual registered entry; the supplied identity itself grants
    /// neither source access nor prepared observation authority.
    pub fn derived_relation(&self, identity: &Identity) -> Option<Handle> {
        self.derived_relations.get(identity).map(|entry| Handle {
            entry: Arc::clone(entry),
        })
    }

    pub(crate) fn admits_derived_relation(&self, handle: &Handle) -> bool {
        self.derived_relations
            .get(&handle.parts().identity)
            .is_some_and(|entry| Arc::ptr_eq(entry, &handle.entry))
    }

    pub(crate) fn derived_parts(&self) -> Vec<Parts> {
        self.derived_relations
            .values()
            .map(|entry| entry.parts.clone())
            .collect()
    }
}
