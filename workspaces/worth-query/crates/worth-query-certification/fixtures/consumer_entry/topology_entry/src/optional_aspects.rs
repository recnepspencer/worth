//! Explicit whole-aspect absence through the consumer declaration facade.
// compiled-example: begin
use worth_query_decl::facade::{
    application_schema::*, worth_query_aspect, worth_query_entity, worth_query_field,
};
pub trait OptionalAspectSchema: ApplicationSchema {}
worth_query_entity!(pub Component for Schema: OptionalAspectSchema);
worth_query_aspect!(pub Identity for Schema: OptionalAspectSchema, Component;
    identity = AspectIdentity(0x9174_2101), revision = AspectContractRevision(1));
worth_query_aspect!(pub Annotation for Schema: OptionalAspectSchema, Component;
    identity = AspectIdentity(0x9174_2102), revision = AspectContractRevision(1));
worth_query_field!(pub Identifier for Schema: OptionalAspectSchema, Component, Identity:
    optional String => StringApplicationValueBinding, read_only, equality);
worth_query_field!(pub Text for Schema: OptionalAspectSchema, Component, Annotation:
    String => StringApplicationValueBinding, read_only, no_equality);

pub fn declare<Schema: OptionalAspectSchema>(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    schema
        .entity(Component::reference())
        // Required whole aspect, despite its optional field.
        .aspect(Component::reference(), Identity::reference())
        .field(Component::reference(), Identifier::reference())
        // Whole absence is legal; a tracked Text read yields None and retains
        // whole-aspect absence. Materializing Annotation invalidates that read.
        // When Annotation is present, Text is required.
        .optional_aspect(Component::reference(), Annotation::reference())
        .field(Component::reference(), Text::reference())
}
// compiled-example: end
