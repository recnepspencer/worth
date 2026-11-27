//! "What references this record?": a governed incoming-link read of one
//! concrete record through an installed query whose root path steps backward
//! from the record along an incoming relation.
//!
//! WORTH declares disclosure per query, not per relation, so a referenced-by
//! read is always an installed query and keeps that query's full admission:
//! principal, scope resolution, disclosure and (for `_governed`) capability.
//! The query is checked against its installed root paths first; one that does
//! not read incoming links of its scoped record is refused before it runs.
//! [`super::WorthQueryEntityImpact::incoming`] lists each incoming relation's
//! installed reverse reads, and so which relations have none.
use worth_query_declaration::facade::{
    application_capability::ApplicationCapabilityRequest,
    application_program::ApplicationProgramDefinition,
    application_query::{
        ApplicationQueryBinding, ApplicationQueryIntent, ApplicationQueryScopeBinding,
        ApplicationQueryScopeResolution,
    },
    application_schema::{
        ApplicationOperationMarkerIdentity, ApplicationSchema, ApplicationStructuredValueBinding,
        RelationDirection,
    },
};
use worth_query_execution::facade::primary_graph::{
    WorthQueryApplicationProjection, WorthQueryPrimaryGraphApplicationRuntime,
};
use worth_query_installation::facade::WorthQueryApplicationQueryInstallationDenial;
use worth_query_publication::facade::application_entry::{
    WorthQueryApplicationRequestQueryDenial, WorthQueryGovernedReadAccess,
};

use super::WorthQueryRelationEdge;
use crate::session::{WorthQueryHostSession, WorthQuerySessionQuery, WorthQuerySessionSnapshot};

type ReadBinding<S, I> = <I as ApplicationQueryIntent<S>>::Binding;
type ReadQuery<S, I> = <ReadBinding<S, I> as ApplicationQueryBinding<S>>::Query;
type ReadPrincipal<S, I> = <ReadBinding<S, I> as ApplicationQueryBinding<S>>::PrincipalIdentity;
type ReadScopeBinding<S, I> = <ReadBinding<S, I> as ApplicationQueryBinding<S>>::ScopeBinding;
type ReadScope<S, I> = <ReadScopeBinding<S, I> as ApplicationQueryScopeBinding<S>>::Scope;
type ReadRow<S, I> = <<ReadBinding<S, I> as ApplicationQueryBinding<S>>::ResultBinding as ApplicationStructuredValueBinding>::Value;

/// The records referencing one record, and the incoming relations read.
#[derive(Clone, Debug)]
pub struct WorthQueryReferencedBy<Row> {
    referenced_entity: String,
    relations: Vec<WorthQueryRelationEdge>,
    rows: Vec<Row>,
}

impl<Row> WorthQueryReferencedBy<Row> {
    /// The referenced record's entity type: the query's scope entity.
    pub fn referenced_entity(&self) -> &str {
        &self.referenced_entity
    }
    /// The incoming relations the query's root paths step backward along.
    pub fn relations(&self) -> &[WorthQueryRelationEdge] {
        &self.relations
    }
    /// The referencing records the principal may see, as the query projects them.
    pub fn rows(&self) -> &[Row] {
        &self.rows
    }
    pub fn into_rows(self) -> Vec<Row> {
        self.rows
    }
}

/// Why a referenced-by read refused. No rows are returned on refusal.
#[derive(Debug)]
pub enum WorthQueryReferencedByDenial {
    /// The query is not installed in this program.
    Installation(WorthQueryApplicationQueryInstallationDenial),
    /// The query has a root path that does not begin by stepping backward
    /// from its scoped record along an incoming relation.
    NotAnIncomingReference { query: String },
    /// The query's own admission, disclosure or capability refused.
    Query(WorthQueryApplicationRequestQueryDenial),
}

impl<Schema, Program> WorthQueryHostSession<'_, Schema, Program>
where
    Schema: ApplicationSchema,
    Program: ApplicationProgramDefinition<Schema>,
{
    /// Reads the records referencing `intent`'s scoped record at the head.
    pub fn referenced_by<Intent: WorthQuerySessionQuery<Schema>>(
        &self,
        intent: Intent,
    ) -> Result<WorthQueryReferencedBy<Intent::Row>, WorthQueryReferencedByDenial> {
        let (referenced_entity, relations) =
            incoming_relations::<Schema, Intent>(self.home.primary_graph())?;
        let rows = self
            .query(intent)
            .map_err(WorthQueryReferencedByDenial::Query)?;
        Ok(WorthQueryReferencedBy {
            referenced_entity,
            relations,
            rows,
        })
    }

    /// [`Self::referenced_by`] for a capability-governed reverse query.
    pub fn referenced_by_governed<Intent, Capability, Operation, Input>(
        &self,
        intent: Intent,
        access: WorthQueryGovernedReadAccess<Schema, Capability, Operation, Input>,
    ) -> Result<WorthQueryReferencedBy<ReadRow<Schema, Intent>>, WorthQueryReferencedByDenial>
    where
        Intent: ApplicationQueryIntent<Schema>,
        ReadScopeBinding<Schema, Intent>:
            ApplicationQueryScopeResolution<Schema, ReadPrincipal<Schema, Intent>>,
        ReadRow<Schema, Intent>: WorthQueryApplicationProjection<Schema, ReadQuery<Schema, Intent>>,
        Operation: ApplicationOperationMarkerIdentity<Schema> + 'static,
        Operation::InputBinding: ApplicationStructuredValueBinding<Value = Input>,
        Input: ApplicationCapabilityRequest<Schema, Capability, Scope = ReadScope<Schema, Intent>>
            + 'static,
    {
        let (referenced_entity, relations) =
            incoming_relations::<Schema, Intent>(self.home.primary_graph())?;
        let (capability, operation, input) = access.into_parts();
        let rows = self
            .query_governed(intent, capability, operation, input)
            .map_err(WorthQueryReferencedByDenial::Query)?;
        Ok(WorthQueryReferencedBy {
            referenced_entity,
            relations,
            rows,
        })
    }
}

impl<Schema: ApplicationSchema, Program> WorthQuerySessionSnapshot<'_, '_, Schema, Program> {
    /// [`WorthQueryHostSession::referenced_by`] at the pinned commit.
    pub fn referenced_by<Intent: WorthQuerySessionQuery<Schema>>(
        &self,
        intent: Intent,
    ) -> Result<WorthQueryReferencedBy<Intent::Row>, WorthQueryReferencedByDenial> {
        let (referenced_entity, relations) =
            incoming_relations::<Schema, Intent>(self.session.home.primary_graph())?;
        let rows = self
            .query(intent)
            .map_err(WorthQueryReferencedByDenial::Query)?;
        Ok(WorthQueryReferencedBy {
            referenced_entity,
            relations,
            rows,
        })
    }
}

/// The scope entity and the incoming relations `Intent`'s installed root
/// paths read, refusing a query with any other root path.
fn incoming_relations<Schema, Intent>(
    application: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
) -> Result<(String, Vec<WorthQueryRelationEdge>), WorthQueryReferencedByDenial>
where
    Schema: ApplicationSchema,
    Intent: ApplicationQueryIntent<Schema>,
{
    let binding = application
        .installed_schema()
        .installed_query_binding::<Intent::Binding>()
        .map_err(WorthQueryReferencedByDenial::Installation)?;
    let query = binding.query();
    let referenced = query.scope_entity().to_owned();
    let not_incoming = || WorthQueryReferencedByDenial::NotAnIncomingReference {
        query: query.name().to_owned(),
    };
    let paths = query.read_graph().root_paths();
    if paths.is_empty() {
        return Err(not_incoming());
    }
    let mut relations = Vec::new();
    for path in paths {
        let first = path.steps().first().ok_or_else(not_incoming)?;
        if first.direction() != RelationDirection::Reverse || first.to() != referenced {
            return Err(not_incoming());
        }
        let edge = WorthQueryRelationEdge::new(first.relation(), first.from(), first.to());
        if !relations.contains(&edge) {
            relations.push(edge);
        }
    }
    relations.sort();
    Ok((referenced, relations))
}
