//! Exact installed projection meaning; admission still belongs to each loan.
use std::sync::Arc;
use worth_foundational::facade::AspectFieldLocator;
use worth_proof::{ManagedIndexSourceAudienceAuthority, ManagedIndexSourceAudienceIssuer};

use crate::derived_relation::{DerivedRelationDenial as Denial, DerivedRelationHandle};
use crate::identity::data::KindId;
use crate::schema::data::RelationalSchemaRegistry;
use crate::validation::data::CustomInvariantAccessContract;

/// A scalar field on the entity endpoint selected by the installed path.
/// Codec and required/optional meaning remain with the Query compiler.
#[derive(Clone, Debug)]
pub struct ManagedIndexSourceField {
    pub locator: AspectFieldLocator,
}

#[derive(Debug)]
pub(crate) struct SourceProgram {
    pub(crate) path: DerivedRelationHandle,
    pub(crate) fields: Arc<[ManagedIndexSourceField]>,
    pub(crate) issuer: ManagedIndexSourceAudienceIssuer,
    pub(crate) authority: ManagedIndexSourceAudienceAuthority,
}

/// A Native schema-bound executable association, never a Source read grant.
#[derive(Clone, Debug)]
pub struct ManagedIndexSourceProgram(pub(crate) Arc<SourceProgram>);

impl PartialEq for ManagedIndexSourceProgram {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}
impl Eq for ManagedIndexSourceProgram {}

impl RelationalSchemaRegistry {
    /// Bind only a recipe actually registered by this schema owner. No root or
    /// caller-supplied scope is admitted by compiling its descriptive meaning.
    pub fn bind_managed_index_source(
        &self,
        path: &DerivedRelationHandle,
        fields: Vec<ManagedIndexSourceField>,
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
        let registration = self
            .entity_kinds
            .get(&row)
            .ok_or(Denial::InvalidPhysicalSource(row))?;
        for field in &fields {
            // The first vertical supports physical scalar fields. Nested or
            // projected endpoints require their own complete inverse program.
            if field.locator.field_path().fields().len() != 1
                || !registration.aspect_contract_declarations.aspects.iter().any(|aspect| {
                    aspect.foundational_key() == field.locator.aspect().aspect_key()
                        && matches!(aspect.contract.shape(), worth_foundational::facade::AspectShape::Struct(shape)
                            if shape.field(&field.locator.field_path().fields()[0]).is_some())
                })
            {
                return Err(Denial::InvalidExpression);
            }
        }
        let issuer = ManagedIndexSourceAudienceIssuer::fresh();
        let authority = issuer.authority();
        Ok(ManagedIndexSourceProgram(Arc::new(SourceProgram {
            path: path.clone(),
            fields: fields.into(),
            issuer,
            authority,
        })))
    }
}

impl ManagedIndexSourceProgram {
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

    pub(crate) fn require_access(
        &self,
        access: &CustomInvariantAccessContract,
    ) -> Result<(), Denial> {
        for kind in [self.scope_kind(), self.row_kind()] {
            if !access.reads_entity(kind) || !access.affects_entity(kind) {
                return Err(Denial::OutsideDeclaredAccess);
            }
        }
        for &(relation, from, to) in self.0.path.physical_sources() {
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
