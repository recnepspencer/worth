use super::{
    ApplicationEntityMarkerIdentity, DottedMemberSchema, IdentifierEntity, InvalidOwnerSchema,
    NamespacedSchema, ProgramEntity, ProgramSchema,
};

impl ApplicationEntityMarkerIdentity<ProgramSchema> for ProgramEntity {
    const IDENTIFIER: &'static str = "ProgramEntity";
}

impl ApplicationEntityMarkerIdentity<NamespacedSchema> for IdentifierEntity {
    const IDENTIFIER: &'static str = "IdentifierEntity";
}

impl ApplicationEntityMarkerIdentity<InvalidOwnerSchema> for IdentifierEntity {
    const IDENTIFIER: &'static str = "IdentifierEntity";
}

impl ApplicationEntityMarkerIdentity<DottedMemberSchema> for IdentifierEntity {
    const IDENTIFIER: &'static str = "Identifier.Entity";
}
