//! Type-declared fragments lowered to the existing flat scalar schema.

mod aspect_selection;
mod catalog;
mod declaration;
mod field_encode_refusal;
mod field_handle;
mod field_selection;
mod identity;
mod lowering;
mod membership;
mod one_of;
pub(crate) mod provenance;
mod record;
mod record_fields;
mod record_optional;
pub(crate) mod record_presence;
mod record_scalar;
mod record_value;
mod references;
mod refusal;
mod relation_field_selection;
mod relation_payload;

pub use aspect_selection::{
    aspect, relation_aspect, AspectSelection, AspectSelector, RelationAspectSelection,
};
pub use catalog::{
    AspectDefinition, FragmentCatalog, FragmentDefinition, RecordFieldDefinition,
    RelationDefinition,
};
pub use declaration::{
    declaration_builder, declare, relation_integrity, visit_correction, Aspect, Entity, Fragment,
    NoBindings, Registration, Relation,
};
pub use field_encode_refusal::FieldEncodeRefusal;
pub use field_handle::{record_field, RecordField};
pub use field_selection::{select_field, FieldSelection, RecordSelection};
#[doc(hidden)]
pub use lowering::{AspectMembers, FragmentMembers, RelationMembers};
#[doc(hidden)]
pub use lowering::{OptionalWitness, RequiredWitness};
pub use membership::{In, Position};
pub use one_of::{CaseDefinition, GroupDefinition};
#[doc(hidden)]
pub use one_of::{CaseRows, CaseWitness, GroupWitness, OptionalCaseWitness, OptionalGroupWitness};
pub use one_of::{Exclusive, OneOf};
pub use provenance::RecordBinding;
pub use record::{field, BindingSet, Record, Scalar, Shape};
pub use references::{entity, relation};
pub use refusal::{RecordRefusal, RecordRefusalKind};
pub use relation_field_selection::{
    select_relation_field, RelationFieldSelection, RelationSelection,
};
pub use relation_payload::WithAspects;
#[doc(hidden)]
pub use relation_payload::{PayloadMembers, PayloadWitness};

pub(in crate::application_schema) use catalog::{expand_record_access, RecordAccess};
use identity::native_aspect_key;
pub(crate) use identity::native_aspect_name;
pub(crate) use record_fields::Columns;
pub(crate) use record_value::ValueColumns;
