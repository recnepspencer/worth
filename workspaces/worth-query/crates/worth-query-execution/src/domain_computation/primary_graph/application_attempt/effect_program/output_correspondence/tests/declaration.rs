//! The test schema, its markers, and the output-role tokens the binding under
//! test declares. A test that needs a token the binding does not declare
//! declares a stray one beside the test, the only way a role reaches Query
//! without being listed.

use worth_query_declaration::facade::application_schema::{
    ApplicationEntityMarkerIdentity, ApplicationSchema, ApplicationSchemaDeclaration,
    ApplicationSchemaDeclarationDenial,
};

use super::super::{
    WorthQueryApplicationOutputRole, WorthQueryCreateOutput, WorthQueryPreserveOutput,
    WorthQueryRetireOutput,
};

pub(super) struct Schema;

impl ApplicationSchema for Schema {
    const OWNER: &'static str = "worth.query.tests";
    const NAME: &'static str = "output-correspondence";
    const MAJOR: u32 = 1;
    const MINOR: u32 = 0;
    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        unreachable!("the output-correspondence proof needs no schema installation")
    }
}

pub(super) struct Binding;
pub(super) struct ForeignBinding;
pub(super) struct Entity;
pub(super) struct WrongEntity;

impl ApplicationEntityMarkerIdentity<Schema> for Entity {
    const IDENTIFIER: &'static str = "entity";
}

impl ApplicationEntityMarkerIdentity<Schema> for WrongEntity {
    const IDENTIFIER: &'static str = "wrong-entity";
}

pub(super) const PRESERVED: WorthQueryApplicationOutputRole<
    Binding,
    Entity,
    WorthQueryPreserveOutput,
> = WorthQueryApplicationOutputRole::for_entity::<Schema>("preserved");
pub(super) const CREATED: WorthQueryApplicationOutputRole<Binding, Entity, WorthQueryCreateOutput> =
    WorthQueryApplicationOutputRole::for_entity::<Schema>("created");
pub(super) const RETIRED: WorthQueryApplicationOutputRole<Binding, Entity, WorthQueryRetireOutput> =
    WorthQueryApplicationOutputRole::for_entity::<Schema>("retired");
