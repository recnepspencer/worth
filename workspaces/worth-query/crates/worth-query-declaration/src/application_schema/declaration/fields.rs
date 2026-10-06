use super::super::aspect_contract_identity::ApplicationAspectMarkerIdentity;
use super::super::capabilities::{
    ApplicationFieldUnit, EqualityCapable, EqualityPosture, WritePosture,
};
use super::super::field_reference::ApplicationFieldRef;
use super::super::references::ApplicationEntityRef;
use super::super::schema_member::ApplicationSchemaMember;
use super::ApplicationSchemaDeclarationBuilder;

impl<Schema> ApplicationSchemaDeclarationBuilder<Schema> {
    pub fn field<Entity, Aspect, Field, Value, Write, Equality, Unit>(
        self,
        entity: ApplicationEntityRef<Schema, Entity>,
        field: ApplicationFieldRef<Schema, Entity, Aspect, Field, Value, Write, Equality, Unit>,
    ) -> Self
    where
        Entity: super::super::ApplicationEntityMarkerIdentity<Schema>,
        Aspect: ApplicationAspectMarkerIdentity<Schema, Entity>,
        Field: super::super::ApplicationFieldMarkerIdentity<Schema, Entity, Aspect, Value = Value>,
        Write: WritePosture,
        Equality: EqualityPosture,
        Unit: ApplicationFieldUnit,
    {
        self.push_field(entity, field, false)
    }

    /// Declares a field no two live entities of its kind may share a value
    /// of. Every program write of the field must observe the value free at
    /// its basis, and a merge must find it free at its target head; the
    /// field's equality index is the lookup.
    pub fn unique_field<Entity, Aspect, Field, Value, Write, Equality, Unit>(
        self,
        entity: ApplicationEntityRef<Schema, Entity>,
        field: ApplicationFieldRef<Schema, Entity, Aspect, Field, Value, Write, Equality, Unit>,
    ) -> Self
    where
        Entity: super::super::ApplicationEntityMarkerIdentity<Schema>,
        Aspect: ApplicationAspectMarkerIdentity<Schema, Entity>,
        Field: super::super::ApplicationFieldMarkerIdentity<Schema, Entity, Aspect, Value = Value>,
        Write: WritePosture,
        Equality: EqualityCapable,
        Unit: ApplicationFieldUnit,
    {
        self.push_field(entity, field, true)
    }

    fn push_field<Entity, Aspect, Field, Value, Write, Equality, Unit>(
        mut self,
        entity: ApplicationEntityRef<Schema, Entity>,
        field: ApplicationFieldRef<Schema, Entity, Aspect, Field, Value, Write, Equality, Unit>,
        unique: bool,
    ) -> Self
    where
        Entity: super::super::ApplicationEntityMarkerIdentity<Schema>,
        Aspect: ApplicationAspectMarkerIdentity<Schema, Entity>,
        Field: super::super::ApplicationFieldMarkerIdentity<Schema, Entity, Aspect, Value = Value>,
        Write: WritePosture,
        Equality: EqualityPosture,
        Unit: ApplicationFieldUnit,
    {
        let recipe = field.binding_recipe();
        self.member_provenance
            .register_field_binding(recipe.clone());
        self.members.push(ApplicationSchemaMember::Field {
            entity: entity.name().to_string(),
            aspect: field.aspect().to_string(),
            field: field.field().to_string(),
            presence: Field::PRESENCE,
            scalar_family: field.scalar_family(),
            value_type: field.value_type_name().to_string(),
            unit: field.unit().map(str::to_string),
            frame: recipe.frame().map(|frame| frame.as_str().to_owned()),
            writable: Write::WRITABLE,
            equality_queryable: Equality::QUERYABLE,
            unique,
        });
        self
    }
}
