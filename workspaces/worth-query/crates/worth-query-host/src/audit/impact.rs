//! "What depends on this entity type?": the installed schema's incoming and
//! outgoing relations of one entity, every capability and ability policy
//! whose grant, scope or authorization path touches it, and the installed
//! queries that read each incoming relation in reverse.
//!
//! This reads only installed declaration meaning, never records, so it
//! discloses no data and needs no principal. Concrete incoming links of one
//! record are read through [`super::WorthQueryReferencedBy`].
use worth_query_declaration::facade::{
    application_capability::ApplicationCapabilityRelationDimension,
    application_schema::{
        ApplicationAuthorizationTraversalDirection, ApplicationEntityRef, ApplicationSchema,
        ApplicationSchemaMember,
    },
};
use worth_query_installation::facade::WorthQueryInstalledApplicationSchema;

use super::reverse_read::{incoming_relations_read, FirstStep};
use crate::session::WorthQueryHostSession;

#[cfg(test)]
mod tests;

/// One declared relation between two entity types.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct WorthQueryRelationEdge {
    relation: String,
    from: String,
    to: String,
}

impl WorthQueryRelationEdge {
    pub(crate) fn new(relation: &str, from: &str, to: &str) -> Self {
        Self {
            relation: relation.to_owned(),
            from: from.to_owned(),
            to: to.to_owned(),
        }
    }
    pub fn relation(&self) -> &str {
        &self.relation
    }
    pub fn from(&self) -> &str {
        &self.from
    }
    pub fn to(&self) -> &str {
        &self.to
    }
    fn touches(&self, entity: &str) -> bool {
        self.from == entity || self.to == entity
    }
}

/// A relation whose target is the inspected entity, with the installed
/// queries whose root path reads it in reverse (its governed referenced-by reads).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryIncomingRelation {
    edge: WorthQueryRelationEdge,
    reverse_reads: Vec<String>,
}

impl WorthQueryIncomingRelation {
    pub fn edge(&self) -> &WorthQueryRelationEdge {
        &self.edge
    }
    /// Installed query names reading this relation in reverse. Empty means no
    /// governed referenced-by read exists for it.
    pub fn reverse_reads(&self) -> &[String] {
        &self.reverse_reads
    }
}

/// How an installed capability or ability policy depends on the entity.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthQueryImpactRole {
    /// Grants of this capability are records of the entity.
    GrantEntity,
    /// The admitted request scope is the entity.
    ScopeEntity,
    /// The authorization policy's principal is the entity.
    PrincipalEntity,
    /// The capability's grant-to-resource head relation touches the entity.
    ResourceHead(WorthQueryRelationEdge),
    /// A capability extent or policy path traverses a relation touching it.
    Traversal {
        edge: WorthQueryRelationEdge,
        direction: ApplicationAuthorizationTraversalDirection,
    },
    /// The capability's bound target relation touches the entity.
    TargetRelation(WorthQueryRelationEdge),
}

/// One installed authority rule depending on the entity.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthQueryImpactDependency {
    Capability {
        capability: String,
        operation: String,
        role: WorthQueryImpactRole,
    },
    AbilityPolicy {
        ability: String,
        scope_entity: String,
        policy: String,
        role: WorthQueryImpactRole,
    },
}

/// Everything installed meaning says depends on one entity type.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryEntityImpact {
    entity: String,
    incoming: Vec<WorthQueryIncomingRelation>,
    outgoing: Vec<WorthQueryRelationEdge>,
    dependencies: Vec<WorthQueryImpactDependency>,
}

impl WorthQueryEntityImpact {
    /// The impact of `entity` in `schema`'s installed declaration.
    pub fn of<Schema: ApplicationSchema, Entity>(
        schema: &WorthQueryInstalledApplicationSchema<Schema>,
        entity: ApplicationEntityRef<Schema, Entity>,
    ) -> Self {
        Self::in_members(schema.installed_declaration().members(), entity.name())
    }

    pub(crate) fn in_members(members: &[ApplicationSchemaMember], entity: &str) -> Self {
        let mut impact = Self {
            entity: entity.to_owned(),
            incoming: Vec::new(),
            outgoing: Vec::new(),
            dependencies: Vec::new(),
        };
        for member in members {
            impact.classify(member);
        }
        for member in members {
            impact.attach_reverse_reads(member);
        }
        impact
    }

    pub fn entity(&self) -> &str {
        &self.entity
    }
    /// Relations whose target is this entity: who can reference it.
    pub fn incoming(&self) -> &[WorthQueryIncomingRelation] {
        &self.incoming
    }
    /// Relations whose source is this entity: what it references.
    pub fn outgoing(&self) -> &[WorthQueryRelationEdge] {
        &self.outgoing
    }
    /// Capabilities and ability policies whose authority depends on it.
    pub fn dependencies(&self) -> &[WorthQueryImpactDependency] {
        &self.dependencies
    }

    fn classify(&mut self, member: &ApplicationSchemaMember) {
        match member {
            ApplicationSchemaMember::Relation {
                relation, from, to, ..
            } => {
                let edge = WorthQueryRelationEdge::new(relation, from, to);
                if *to == self.entity {
                    self.incoming.push(WorthQueryIncomingRelation {
                        edge: edge.clone(),
                        reverse_reads: Vec::new(),
                    });
                }
                if *from == self.entity {
                    self.outgoing.push(edge);
                }
            }
            ApplicationSchemaMember::ApplicationCapability { contract } => {
                for role in self.capability_roles(contract) {
                    self.dependencies
                        .push(WorthQueryImpactDependency::Capability {
                            capability: contract.name().to_owned(),
                            operation: contract.operation().to_owned(),
                            role,
                        });
                }
            }
            ApplicationSchemaMember::AbilityPolicy {
                ability,
                scope_entity,
                policy,
                paths,
            } => {
                let mut roles = Vec::new();
                if *scope_entity == self.entity {
                    roles.push(WorthQueryImpactRole::ScopeEntity);
                }
                for path in paths {
                    if path.principal_entity() == self.entity {
                        roles.push(WorthQueryImpactRole::PrincipalEntity);
                    }
                    roles.extend(self.traversal_roles(path.traversals()));
                }
                self.dependencies
                    .extend(distinct(roles).into_iter().map(|role| {
                        WorthQueryImpactDependency::AbilityPolicy {
                            ability: ability.clone(),
                            scope_entity: scope_entity.clone(),
                            policy: policy.clone(),
                            role,
                        }
                    }));
            }
            _ => {}
        }
    }

    fn capability_roles(
        &self,
        contract: &worth_query_declaration::facade::application_capability::ErasedApplicationCapabilityContract,
    ) -> Vec<WorthQueryImpactRole> {
        let mut roles = Vec::new();
        if contract.grant_entity() == self.entity {
            roles.push(WorthQueryImpactRole::GrantEntity);
        }
        let extent = contract.target().resource_path();
        if extent.scope_entity() == self.entity {
            roles.push(WorthQueryImpactRole::ScopeEntity);
        }
        let head = extent.head();
        let head = WorthQueryRelationEdge::new(head.relation(), head.from(), head.to());
        if head.touches(&self.entity) {
            roles.push(WorthQueryImpactRole::ResourceHead(head));
        }
        roles.extend(self.traversal_roles(extent.traversals()));
        if let ApplicationCapabilityRelationDimension::Bound(target) = contract.target().relation()
        {
            let target = WorthQueryRelationEdge::new(target.relation(), target.from(), target.to());
            if target.touches(&self.entity) {
                roles.push(WorthQueryImpactRole::TargetRelation(target));
            }
        }
        distinct(roles)
    }

    fn traversal_roles<'path>(
        &'path self,
        traversals: &'path [worth_query_declaration::facade::application_schema::ApplicationAuthorizationTraversal],
    ) -> impl Iterator<Item = WorthQueryImpactRole> + 'path {
        traversals.iter().filter_map(|traversal| {
            let edge =
                WorthQueryRelationEdge::new(traversal.relation(), traversal.from(), traversal.to());
            edge.touches(&self.entity)
                .then(|| WorthQueryImpactRole::Traversal {
                    edge,
                    direction: traversal.direction(),
                })
        })
    }

    /// Records each installed referenced-by read of this entity, by the
    /// same rule `referenced_by` admits it with.
    fn attach_reverse_reads(&mut self, member: &ApplicationSchemaMember) {
        let ApplicationSchemaMember::ApplicationQuery { definition } = member else {
            return;
        };
        let first_steps = definition.root_paths().iter().map(|path| {
            path.steps().first().map(|step| FirstStep {
                direction: step.direction(),
                relation: step.relation(),
                from: step.from(),
                to: step.to(),
            })
        });
        self.attach_query(definition.name(), definition.scope_entity(), first_steps);
    }

    fn attach_query<'query>(
        &mut self,
        query: &str,
        scope_entity: &str,
        first_steps: impl IntoIterator<Item = Option<FirstStep<'query>>>,
    ) {
        if scope_entity != self.entity {
            return;
        }
        let Some(edges) = incoming_relations_read(scope_entity, first_steps) else {
            return;
        };
        for edge in edges {
            if let Some(incoming) = self
                .incoming
                .iter_mut()
                .find(|incoming| incoming.edge == edge)
                && !incoming.reverse_reads.iter().any(|name| name == query)
            {
                incoming.reverse_reads.push(query.to_owned());
            }
        }
    }
}

/// Each role once, in first-occurrence order: two paths of one policy (or a
/// path and its contract's head) may reach the entity the same way.
fn distinct(roles: Vec<WorthQueryImpactRole>) -> Vec<WorthQueryImpactRole> {
    let mut seen: Vec<WorthQueryImpactRole> = Vec::with_capacity(roles.len());
    for role in roles {
        if !seen.contains(&role) {
            seen.push(role);
        }
    }
    seen
}

impl<Schema: ApplicationSchema, Program> WorthQueryHostSession<'_, Schema, Program> {
    /// [`WorthQueryEntityImpact::of`] in this session's installed schema.
    pub fn entity_impact<Entity>(
        &self,
        entity: ApplicationEntityRef<Schema, Entity>,
    ) -> WorthQueryEntityImpact {
        WorthQueryEntityImpact::of(self.home.primary_graph().installed_schema(), entity)
    }
}
