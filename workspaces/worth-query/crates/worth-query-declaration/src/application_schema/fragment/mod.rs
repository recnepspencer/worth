//! Type-declared fragments lowered to the existing flat scalar schema.

mod catalog;
mod declaration;
mod field_encode_refusal;
mod field_handle;
mod identity;
mod lowering;
mod membership;
mod one_of;
pub(crate) mod provenance;
mod record;
mod record_fields;
mod record_value;
mod references;
mod refusal;

pub use catalog::{
    AspectDefinition, FragmentCatalog, FragmentDefinition, RecordFieldDefinition,
    RelationDefinition,
};
pub use declaration::{declare, Aspect, Entity, Fragment, Registration, Relation};
pub use field_encode_refusal::FieldEncodeRefusal;
pub use field_handle::{record_field, RecordField};
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

pub(crate) use catalog::expand_record_access;
pub(crate) use record_fields::Columns;
pub(crate) use record_value::ValueColumns;
