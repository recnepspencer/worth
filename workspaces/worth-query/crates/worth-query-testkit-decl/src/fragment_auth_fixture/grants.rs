//! Installed principal, ability and operation contracts for the fragment court.
use super::*;
use worth_query_decl::facade::{
    worth_query_operation, worth_query_operation_requires, worth_query_policy,
};

pub type IdentityValueBinding = <Bindings as Scalar<u64>>::Binding;
pub struct IdentityBinding;
impl IdentityBinding {
    pub fn reference() -> Result<
        ApplicationPrincipalBindingRef<
            Fixture,
            Self,
            Mapping,
            Principal,
            u64,
            IdentityValueBinding,
        >,
        ApplicationSchemaDeclarationDenial,
    > {
        Ok(ApplicationPrincipalBindingRef::from_requirements(
            "identity",
            ApplicationPrincipalBindingRequirements {
                mapping_identity: ApplicationPrincipalMappingIdentityRequirement::from_record(
                    record_field(Authentication, Mapping, MappingIdentity, "external")?,
                )?,
                mapping_status: ApplicationPrincipalMappingStatusRequirement::from_record(
                    record_field(Authentication, Mapping, MappingIdentity, "status")?,
                )?,
                target: ApplicationPrincipalTargetRequirement::from_relation(relation(
                    Authentication,
                    Mapping,
                    MapsTo,
                )),
                principal_identity: ApplicationPrincipalIdentityRequirement::from_record(
                    Authentication,
                    record_field(Authentication, Principal, Identity, "id")?,
                )?,
            },
        ))
    }
}
impl Registration<Fixture> for IdentityBinding {
    fn register(
        builder: ApplicationSchemaDeclarationBuilder<Fixture>,
    ) -> Result<ApplicationSchemaDeclarationBuilder<Fixture>, ApplicationSchemaDeclarationDenial>
    {
        Ok(builder.principal_binding(Self::reference()?))
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Edit;
impl Edit {
    pub const fn reference() -> ApplicationAbilityRef<Fixture, Self, Item> {
        ApplicationAbilityRef::from_schema_identifiers("Edit", <Item as Entity>::NAME)
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Inspect;
impl Inspect {
    pub const fn reference() -> ApplicationAbilityRef<Fixture, Self, OptionalItem> {
        ApplicationAbilityRef::from_schema_identifiers("Inspect", <OptionalItem as Entity>::NAME)
    }
}
worth_query_policy!(pub ItemAccess in Fixture);
worth_query_policy!(pub OptionalAccess in Fixture);
pub struct Access;
impl Registration<Fixture> for Access {
    fn register(
        builder: ApplicationSchemaDeclarationBuilder<Fixture>,
    ) -> Result<ApplicationSchemaDeclarationBuilder<Fixture>, ApplicationSchemaDeclarationDenial>
    {
        Ok(builder
            .ability(Edit::reference())
            .policy(ItemAccess::reference())
            .ability_policy(
                Edit::reference(),
                ItemAccess::reference(),
                [
                    ApplicationAuthorizationPathBuilder::<Fixture, Principal>::from_principal(
                        entity(Authentication, Principal),
                    )
                    .forward(relation(Authentication, Principal, OwnsItem))
                    .allow(entity(Fixture, Item)),
                ],
            )
            .ability(Inspect::reference())
            .policy(OptionalAccess::reference())
            .ability_policy(
                Inspect::reference(),
                OptionalAccess::reference(),
                [
                    ApplicationAuthorizationPathBuilder::<Fixture, Principal>::from_principal(
                        entity(Authentication, Principal),
                    )
                    .forward(relation(Authentication, Principal, OwnsOptional))
                    .allow(entity(Authentication, OptionalItem)),
                ],
            ))
    }
}
worth_query_operation!(pub ChangeForm for Fixture, input UnitInput);
worth_query_operation_requires!(ChangeForm => [Edit]);
impl OperationReads<ChangeForm> for Item {}
impl OperationCreates<ChangeForm> for Item {}
impl Registration<Fixture> for ChangeForm {
    fn register(
        builder: ApplicationSchemaDeclarationBuilder<Fixture>,
    ) -> Result<ApplicationSchemaDeclarationBuilder<Fixture>, ApplicationSchemaDeclarationDenial>
    {
        let operation = Self::reference();
        Ok(builder
            .operation(
                operation
                    .definition()
                    .no_external_effect()
                    .no_aftermath()
                    .finish(),
            )
            .operation_requires_ability(operation, Edit::reference())
            .operation_read_entity(operation, entity(Fixture, Item))
            .operation_read_record(operation, Fixture, Item, ItemIdentity)
            .operation_read_record(operation, Fixture, Item, Form)
            .operation_read_record(operation, Fixture, Item, Note)
            .operation_read_record(operation, Fixture, Item, RequiredNote)
            .operation_create(operation, entity(Fixture, Item))
            .operation_write_record(operation, Fixture, Item, ItemIdentity)
            .operation_write_record(operation, Fixture, Item, Metrics)
            .operation_write_record(operation, Fixture, Item, Noted)
            .operation_write_record(operation, Fixture, Item, Note)
            .operation_write_record(operation, Fixture, Item, RequiredNote)
            .operation_write_record(operation, Fixture, Item, Named)
            .operation_write_record(operation, Fixture, Item, Measured))
    }
}

worth_query_operation!(pub FieldsOnly for Fixture, input UnitInput);
worth_query_operation_requires!(FieldsOnly => [Edit]);
impl OperationReads<FieldsOnly> for Item {}
impl Registration<Fixture> for FieldsOnly {
    fn register(
        builder: ApplicationSchemaDeclarationBuilder<Fixture>,
    ) -> Result<ApplicationSchemaDeclarationBuilder<Fixture>, ApplicationSchemaDeclarationDenial>
    {
        let operation = Self::reference();
        let selected: RecordField<_, _, String> = record_field(Fixture, Item, Named, "text")?;
        Ok(builder
            .operation(
                operation
                    .definition()
                    .no_external_effect()
                    .no_aftermath()
                    .finish(),
            )
            .operation_requires_ability(operation, Edit::reference())
            .operation_read_entity(operation, entity(Fixture, Item))
            .operation_read_record(operation, Fixture, Item, Form)
            .operation_write_record_field(operation, selected))
    }
}
worth_query_operation!(pub InspectOptional for Fixture, input UnitInput);
worth_query_operation_requires!(InspectOptional => [Inspect]);
impl OperationReads<InspectOptional> for OptionalItem {}
impl Registration<Fixture> for InspectOptional {
    fn register(
        builder: ApplicationSchemaDeclarationBuilder<Fixture>,
    ) -> Result<ApplicationSchemaDeclarationBuilder<Fixture>, ApplicationSchemaDeclarationDenial>
    {
        let operation = Self::reference();
        Ok(builder
            .operation(
                operation
                    .definition()
                    .no_external_effect()
                    .no_aftermath()
                    .finish(),
            )
            .operation_requires_ability(operation, Inspect::reference())
            .operation_read_entity(operation, entity(Authentication, OptionalItem))
            .operation_read_record(operation, Authentication, OptionalItem, OptionalIdentity)
            .operation_read_record(operation, Authentication, OptionalItem, OptionalForm))
    }
}
