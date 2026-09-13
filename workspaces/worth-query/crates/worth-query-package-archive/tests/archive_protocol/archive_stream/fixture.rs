use worth_query_declaration::facade::application_schema::{
    ApplicationOperationMarkerIdentity, ApplicationOperationRef, ApplicationSchema,
    ApplicationSchemaDeclaration, ApplicationSchemaDeclarationBuilder,
    StringApplicationValueBinding,
};
use worth_query_installation::facade::{
    WorthQueryPortableApplicationConditionalOperationBinding,
    WorthQueryPortableApplicationConditionalOperationBindingParts, WorthQueryPortableDefinition,
    WorthQueryPortableDomainIdentity, WorthQueryPortableDomainOperationDefinition,
    WorthQueryPortableDomainPackage, WorthQueryValidatedPortableDomainPackage,
};

struct CompleteSchema;
struct CompleteOperation;
struct CompleteInput;

worth_query_declaration::worth_query_portable_type!(
    CompleteInput => "worth.query.archive.stream.input"
);
worth_query_declaration::worth_query_entity!(CompleteEntity for CompleteSchema);
worth_query_declaration::worth_query_aspect!(
    Profile for CompleteSchema, CompleteEntity;
    identity = AspectIdentity(0x9162_1300),
    revision = AspectContractRevision(1),
);
worth_query_declaration::worth_query_field!(
    Name for CompleteSchema, CompleteEntity, Profile:
    String => StringApplicationValueBinding, read_only, equality
);
worth_query_declaration::worth_query_structured_value_binding!(
    CompleteInputBinding for CompleteInput {
        identity: "worth.query.archive.stream.input"
    }
);

impl ApplicationOperationMarkerIdentity<CompleteSchema> for CompleteOperation {
    type InputBinding = CompleteInputBinding;
    const IDENTIFIER: &'static str = "ArchiveStreamOperation";
}

impl ApplicationSchema for CompleteSchema {
    const OWNER: &'static str = "archive.stream.complete";
    const NAME: &'static str = "ArchiveStreamSchema";
    const MAJOR: u32 = 1;
    const MINOR: u32 = 0;

    fn declaration() -> Result<
        ApplicationSchemaDeclaration<Self>,
        worth_query_declaration::facade::application_schema::ApplicationSchemaDeclarationDenial,
    > {
        let operation =
            ApplicationOperationRef::<Self, CompleteOperation, CompleteInput>::from_declaration();
        ApplicationSchemaDeclarationBuilder::for_schema()
            .entity(CompleteEntity::reference())
            .aspect(CompleteEntity::reference(), Profile::reference())
            .field(CompleteEntity::reference(), Name::reference())
            .operation(
                operation
                    .definition()
                    .no_external_effect()
                    .no_aftermath()
                    .finish(),
            )
            .build()
    }
}

pub(crate) fn minimal_package() -> WorthQueryValidatedPortableDomainPackage {
    WorthQueryPortableDomainPackage::new(WorthQueryPortableDomainIdentity::new(
        "archive.stream.minimal",
        1,
        0,
    ))
    .validate()
    .unwrap()
}

pub(crate) fn all_family_package() -> WorthQueryValidatedPortableDomainPackage {
    let operation = super::super::domain_operation_record::fixture::operation();
    let binding = conditional_binding(&operation);
    WorthQueryPortableDomainPackage::new(WorthQueryPortableDomainIdentity::new(
        CompleteSchema::OWNER,
        1,
        0,
    ))
    .requires_capability("query-read")
    .requires_configuration("query")
    .requires_operating_posture("bounded")
    .definition(WorthQueryPortableDefinition::invariant(
        "connected",
        "one-outgoing",
    ))
    .domain_operation(operation)
    .artifact_contract(super::super::artifact_contract_record::fixture::artifact_contract())
    .application_schema(CompleteSchema::declaration().unwrap())
    .conditional_application_operation_erased(binding)
    .permits_contribution("query-index")
    .validate()
    .unwrap()
}

pub(crate) fn ordered_requirement_package() -> WorthQueryValidatedPortableDomainPackage {
    WorthQueryPortableDomainPackage::new(WorthQueryPortableDomainIdentity::new(
        "archive.stream.requirements",
        1,
        0,
    ))
    .requires_capability("alpha")
    .requires_capability("omega")
    .validate()
    .unwrap()
}

fn conditional_binding(
    operation: &WorthQueryPortableDomainOperationDefinition,
) -> WorthQueryPortableApplicationConditionalOperationBinding {
    WorthQueryPortableApplicationConditionalOperationBinding::from_untrusted_parts(
        WorthQueryPortableApplicationConditionalOperationBindingParts {
            schema_owner: CompleteSchema::OWNER.to_owned(),
            schema_name: CompleteSchema::NAME.to_owned(),
            application_operation: CompleteOperation::IDENTIFIER.to_owned(),
            input_type: worth_query_declaration::facade::portable_identity::WorthQueryPortableTypeIdentity::from_untrusted(
                "worth.query.archive.stream.input".to_owned(),
            ),
            domain_operation_slot: operation.identity().slot(),
            domain_operation_canonical_identity: operation.canonical_identity().to_owned(),
        },
    )
}
