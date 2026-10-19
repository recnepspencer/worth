//! Whole-aspect absence is independent of every field's presence law.
use super::super::{ApplicationAspectMarkerIdentity, ApplicationAspectRef, ApplicationEntityRef};
use super::{ApplicationSchemaDeclarationBuilder, ApplicationSchemaMember};
use worth_foundational::facade::AbsenceLaw;

impl<Schema> ApplicationSchemaDeclarationBuilder<Schema> {
    /// Declares an aspect that must be present, even when all its fields are optional.
    pub fn aspect<Entity, Aspect>(
        self,
        entity: ApplicationEntityRef<Schema, Entity>,
        aspect: ApplicationAspectRef<Schema, Entity, Aspect>,
    ) -> Self
    where
        Aspect: ApplicationAspectMarkerIdentity<Schema, Entity>,
    {
        self.aspect_with_absence(entity, aspect, AbsenceLaw::Required)
    }
    /// Permits the entire aspect to be absent. If present, its field requirements still apply.
    pub fn optional_aspect<Entity, Aspect>(
        self,
        entity: ApplicationEntityRef<Schema, Entity>,
        aspect: ApplicationAspectRef<Schema, Entity, Aspect>,
    ) -> Self
    where
        Aspect: ApplicationAspectMarkerIdentity<Schema, Entity>,
    {
        self.aspect_with_absence(entity, aspect, AbsenceLaw::Optional)
    }
    fn aspect_with_absence<Entity, Aspect>(
        mut self,
        entity: ApplicationEntityRef<Schema, Entity>,
        aspect: ApplicationAspectRef<Schema, Entity, Aspect>,
        absence: AbsenceLaw,
    ) -> Self
    where
        Aspect: ApplicationAspectMarkerIdentity<Schema, Entity>,
    {
        self.members.push(ApplicationSchemaMember::Aspect {
            entity: entity.name().to_owned(),
            aspect: aspect.name().to_owned(),
            identity: Aspect::ASPECT_IDENTITY,
            revision: Aspect::CONTRACT_REVISION,
            absence,
        });
        self
    }
}
