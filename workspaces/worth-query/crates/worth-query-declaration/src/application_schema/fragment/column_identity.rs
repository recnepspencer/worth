//! Marker facts carried to installed column lookup without generic traversal.
use super::{FragmentCatalog, RecordFieldDefinition};
use crate::application_schema::{
    ApplicationFieldBindingLocus as Locus, ApplicationFieldBindingOwnerKind as Owner,
    ApplicationSchemaDeclarationDenial as Denial,
};
use std::any::TypeId;

#[doc(hidden)]
#[derive(Clone, Copy)]
pub struct ColumnIdentity {
    pub(super) fragment: TypeId,
    pub(super) record: TypeId,
    pub(super) aspect: TypeId,
    pub(super) value: TypeId,
    pub(super) binding: TypeId,
    pub(super) fragment_name: &'static str,
    pub(super) record_name: &'static str,
    pub(super) aspect_name: &'static str,
    pub(super) native_aspect: &'static str,
    pub(super) field: &'static str,
    pub(super) owner: Owner,
}
impl ColumnIdentity {
    pub fn entity(self) -> &'static str {
        self.record_name
    }
    pub fn aspect(self) -> &'static str {
        self.native_aspect
    }
    pub fn field(self) -> &'static str {
        self.field
    }
    pub fn owner(self) -> Owner {
        self.owner
    }
    pub fn locus(self) -> Locus {
        match self.owner {
            Owner::Entity => Locus::new(self.record_name, self.native_aspect, self.field),
            Owner::Relation => {
                Locus::for_relation(self.record_name, self.native_aspect, self.field)
            }
        }
    }
    pub fn installed(self, catalog: &FragmentCatalog) -> Result<&RecordFieldDefinition, Denial> {
        catalog
            .column(self)
            .ok_or_else(|| Denial::InvalidRecordField {
                field: self.locus(),
            })
    }
}
