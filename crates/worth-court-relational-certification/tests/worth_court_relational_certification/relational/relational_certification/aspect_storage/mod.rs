//! Installed scalar bindings and their actual canonical publication effects.
use worth_foundational::facade::{
    AspectBinding, ContractValidationInput, PortableAspectContractBasis,
    PortableAspectPatchOperation, PortableRecordAspectPatch, StructAspectValue,
};
use worth_relational::facade::{
    identity::{EntityId, KindId, PartitionId},
    runtime::ProjectionAspectScope,
    schema::DeclaredAspectContractBinding,
    transactions::*,
};
use worth_testkit_relational::aspect_storage::{
    aspect_field_locator, aspect_key, entity_field_aspect, entity_summary_struct_aspect, field_key,
    optional_string_contract, publications::*, AspectSchemaFixture,
};
mod canonical_field_effect;
mod scalar_fields;
