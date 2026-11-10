//! Exact installed projection meaning; admission still belongs to each loan.
use std::sync::Arc;
use worth_foundational::facade::AspectFieldLocator;
use worth_proof::{ManagedIndexSourceAudienceAuthority, ManagedIndexSourceAudienceIssuer};

use crate::derived_relation::{DerivedRelationDenial as Denial, DerivedRelationHandle};
use crate::identity::data::KindId;
use crate::schema::data::RelationalSchemaRegistry;
use crate::validation::data::CustomInvariantAccessContract;

mod bindings;
mod ordered_predecessor;
pub(crate) use bindings::SourceBindingCache;
mod projection_routes;
pub use projection_routes::ManagedIndexSourceProjectionRoute;

/// A scalar field on the entity endpoint selected by the installed path.
/// Codec and required/optional meaning remain with the Query compiler.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ManagedIndexSourceField {
    pub locator: AspectFieldLocator,
}

pub(crate) struct SourceProgram {
    // In-memory bind order only; never persist, content-hash, or include in evidence.
    order: u64,
    pub(crate) path: DerivedRelationHandle,
    pub(crate) ordered: Option<ordered_predecessor::OrderedSourceRecipe>,
    pub(crate) fields: Arc<[ManagedIndexSourceField]>,
    pub(crate) routes: Arc<[ManagedIndexSourceProjectionRoute]>,
    pub(crate) issuer: ManagedIndexSourceAudienceIssuer,
    pub(crate) authority: ManagedIndexSourceAudienceAuthority,
}

/// A Native schema-bound executable association, never a Source read grant.
#[derive(Clone, Debug)]
pub struct ManagedIndexSourceProgram(pub(crate) Arc<SourceProgram>);

impl PartialEq for ManagedIndexSourceProgram {
    fn eq(&self, other: &Self) -> bool {
        self.0.order == other.0.order
    }
}
impl Eq for ManagedIndexSourceProgram {}
impl Ord for ManagedIndexSourceProgram {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.order.cmp(&other.0.order)
    }
}
impl PartialOrd for ManagedIndexSourceProgram {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl std::fmt::Debug for SourceProgram {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("SourceProgram")
            .field("path", &self.path)
            .field("fields", &self.fields)
            .field("routes", &self.routes)
            .field("issuer", &self.issuer)
            .field("authority", &self.authority)
            .finish_non_exhaustive()
    }
}

impl RelationalSchemaRegistry {
    /// Bind only a recipe actually registered by this schema owner. No root or
    /// caller-supplied scope is admitted by compiling its descriptive meaning.
    pub fn bind_managed_index_source(
        &self,
        path: &DerivedRelationHandle,
        fields: Vec<ManagedIndexSourceField>,
        routes: Vec<ManagedIndexSourceProjectionRoute>,
    ) -> Result<ManagedIndexSourceProgram, Denial> {
        if !self.admits_derived_relation(path) {
            return Err(Denial::ForeignDefinition);
        }
        let row = path
            .parts()
            .to
            .entity_kind()
            .ok_or(Denial::InvalidMemberDomain)?;
        path.parts()
            .from
            .entity_kind()
            .ok_or(Denial::InvalidMemberDomain)?;
        projection_routes::validate_fields(self, row, &fields)?;
        projection_routes::validate_routes(self, row, &routes)?;
        projection_routes::validate_route_names(&routes)?;
        path.entry
            .managed_source_binding
            .bind(path, fields, routes, None)
    }
}

impl ManagedIndexSourceProgram {
    pub fn has_ordered_predecessor(&self) -> bool {
        self.0.ordered.is_some()
    }

    pub fn scope_kind(&self) -> KindId {
        self.0
            .path
            .parts()
            .from
            .entity_kind()
            .expect("compiled physical scope")
    }

    pub fn row_kind(&self) -> KindId {
        self.0
            .path
            .parts()
            .to
            .entity_kind()
            .expect("compiled physical row")
    }

    pub fn fields(&self) -> &[ManagedIndexSourceField] {
        &self.0.fields
    }

    pub fn projection_routes(&self) -> &[ManagedIndexSourceProjectionRoute] {
        &self.0.routes
    }

    pub(crate) fn paths(&self) -> impl Iterator<Item = &DerivedRelationHandle> {
        std::iter::once(&self.0.path).chain(self.0.routes.iter().map(|route| &route.path))
    }

    pub(crate) fn physical_sources(&self) -> impl Iterator<Item = &(KindId, KindId, KindId)> {
        self.paths().flat_map(|path| path.physical_sources())
    }

    pub(crate) fn require_registry(
        &self,
        registry: &RelationalSchemaRegistry,
    ) -> Result<(), Denial> {
        if self
            .paths()
            .any(|path| !registry.admits_derived_relation(path))
        {
            return Err(Denial::ForeignDefinition);
        }
        projection_routes::validate_fields(registry, self.row_kind(), self.fields())?;
        projection_routes::validate_routes(registry, self.row_kind(), self.projection_routes())
    }

    pub(crate) fn require_access(
        &self,
        access: &CustomInvariantAccessContract,
    ) -> Result<(), Denial> {
        for kind in [self.scope_kind(), self.row_kind()] {
            if !access.reads_entity(kind) || !access.affects_entity(kind) {
                return Err(Denial::OutsideDeclaredAccess);
            }
        }
        for &(relation, from, to) in self.physical_sources() {
            if !access.reads_relation(relation)
                || !access.affects_relation(relation)
                || !access.reads_entity(from)
                || !access.affects_entity(from)
                || !access.reads_entity(to)
                || !access.affects_entity(to)
            {
                return Err(Denial::OutsideDeclaredAccess);
            }
        }
        Ok(())
    }
}
