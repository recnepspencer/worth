use worth_foundational::facade::{AspectContractRevision, AspectIdentity};
use worth_query_declaration::facade::application_schema::{
    ApplicationAspectMarkerIdentity, ApplicationAspectRef, ApplicationFieldMarkerIdentity,
    ApplicationFieldPresence, ApplicationFieldRef, ApplicationSchema, ApplicationSchemaDeclaration,
    ApplicationSchemaDeclarationBuilder, ApplicationSchemaDeclarationDenial,
    DeclaredApplicationFieldValue, StringApplicationValueBinding, U64ApplicationValueBinding,
};

use super::{assert_package_contract_denial, WorthQueryPortablePackageValidationDenialKind};

struct DuplicateAspectLocusSchema;
struct DuplicateFieldLocusSchema;
struct DuplicateIdentityAcrossEntitiesSchema;
struct SameNameAcrossEntitiesSchema;
worth_query_declaration::worth_query_entity!(AspectEntity for DuplicateAspectLocusSchema);
worth_query_declaration::worth_query_entity!(FieldEntity for DuplicateFieldLocusSchema);
worth_query_declaration::worth_query_entity!(FirstIdentityEntity for DuplicateIdentityAcrossEntitiesSchema);
worth_query_declaration::worth_query_entity!(SecondIdentityEntity for DuplicateIdentityAcrossEntitiesSchema);
worth_query_declaration::worth_query_entity!(FirstNamedEntity for SameNameAcrossEntitiesSchema);
worth_query_declaration::worth_query_entity!(SecondNamedEntity for SameNameAcrossEntitiesSchema);
worth_query_declaration::worth_query_aspect!(
    FieldAspect for DuplicateFieldLocusSchema, FieldEntity;
    identity = AspectIdentity(21),
    revision = AspectContractRevision(1),
);

struct FirstAspect;
struct SecondAspect;
struct FirstField;
struct SecondField;
struct FirstIdentityAspect;
struct SecondIdentityAspect;
struct FirstNamedAspect;
struct SecondNamedAspect;
struct FirstNamedField;
struct SecondNamedField;

macro_rules! duplicate_aspect_marker {
    ($marker:ty, $identity:expr) => {
        impl ApplicationAspectMarkerIdentity<DuplicateAspectLocusSchema, AspectEntity> for $marker {
            const IDENTIFIER: &'static str = "SameAspect";
            const ASPECT_IDENTITY: AspectIdentity = AspectIdentity($identity);
            const CONTRACT_REVISION: AspectContractRevision = AspectContractRevision(1);
        }
    };
}

duplicate_aspect_marker!(FirstAspect, 11);
duplicate_aspect_marker!(SecondAspect, 12);

macro_rules! duplicate_field_marker {
    ($marker:ty, $value:ty, $binding:ty) => {
        impl ApplicationFieldMarkerIdentity<DuplicateFieldLocusSchema, FieldEntity, FieldAspect>
            for $marker
        {
            const IDENTIFIER: &'static str = "SameField";
        }

        impl DeclaredApplicationFieldValue for $marker {
            type Value = $value;
            type Binding = $binding;
            const PRESENCE: ApplicationFieldPresence = ApplicationFieldPresence::Required;
        }
    };
}

duplicate_field_marker!(FirstField, u64, U64ApplicationValueBinding);
duplicate_field_marker!(SecondField, String, StringApplicationValueBinding);

macro_rules! aspect_marker {
    ($marker:ty, $schema:ty, $entity:ty, $name:literal, $identity:expr) => {
        impl ApplicationAspectMarkerIdentity<$schema, $entity> for $marker {
            const IDENTIFIER: &'static str = $name;
            const ASPECT_IDENTITY: AspectIdentity = AspectIdentity($identity);
            const CONTRACT_REVISION: AspectContractRevision = AspectContractRevision(1);
        }
    };
}

aspect_marker!(
    FirstIdentityAspect,
    DuplicateIdentityAcrossEntitiesSchema,
    FirstIdentityEntity,
    "FirstAspect",
    31
);
aspect_marker!(
    SecondIdentityAspect,
    DuplicateIdentityAcrossEntitiesSchema,
    SecondIdentityEntity,
    "SecondAspect",
    31
);
aspect_marker!(
    FirstNamedAspect,
    SameNameAcrossEntitiesSchema,
    FirstNamedEntity,
    "SameAspect",
    41
);
aspect_marker!(
    SecondNamedAspect,
    SameNameAcrossEntitiesSchema,
    SecondNamedEntity,
    "SameAspect",
    42
);

macro_rules! named_field_marker {
    ($marker:ty, $entity:ty, $aspect:ty) => {
        impl ApplicationFieldMarkerIdentity<SameNameAcrossEntitiesSchema, $entity, $aspect>
            for $marker
        {
            const IDENTIFIER: &'static str = "SameField";
        }
        impl DeclaredApplicationFieldValue for $marker {
            type Value = u64;
            type Binding = U64ApplicationValueBinding;
            const PRESENCE: ApplicationFieldPresence = ApplicationFieldPresence::Required;
        }
    };
}

named_field_marker!(FirstNamedField, FirstNamedEntity, FirstNamedAspect);
named_field_marker!(SecondNamedField, SecondNamedEntity, SecondNamedAspect);

macro_rules! schema_identity {
    ($schema:ty, $name:literal) => {
        impl ApplicationSchema for $schema {
            const OWNER: &'static str = "native-contract-semantic-collision";
            const NAME: &'static str = $name;
            const MAJOR: u32 = 1;
            const MINOR: u32 = 0;

            fn declaration() -> Result<
                ApplicationSchemaDeclaration<Self>,
                worth_query_declaration::facade::application_schema::ApplicationSchemaDeclarationDenial,
            > {
                unreachable!("the collision tests author exact declarations")
            }
        }
    };
}

schema_identity!(DuplicateAspectLocusSchema, "DuplicateAspectLocusSchema");
schema_identity!(DuplicateFieldLocusSchema, "DuplicateFieldLocusSchema");
schema_identity!(
    DuplicateIdentityAcrossEntitiesSchema,
    "DuplicateIdentityAcrossEntitiesSchema"
);
impl ApplicationSchema for SameNameAcrossEntitiesSchema {
    const OWNER: &'static str = "native-contract-semantic-collision";
    const NAME: &'static str = "SameNameAcrossEntitiesSchema";
    const MAJOR: u32 = 1;
    const MINOR: u32 = 0;

    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        let first = FirstNamedEntity::reference();
        let second = SecondNamedEntity::reference();
        ApplicationSchemaDeclarationBuilder::for_schema()
            .entity(first)
            .entity(second)
            .aspect(
                first,
                ApplicationAspectRef::<_, _, FirstNamedAspect>::from_schema_identifier(
                    "SameAspect",
                ),
            )
            .aspect(
                second,
                ApplicationAspectRef::<_, _, SecondNamedAspect>::from_schema_identifier(
                    "SameAspect",
                ),
            )
            .field(
                first,
                ApplicationFieldRef::<_, _, _, FirstNamedField, u64>::from_schema_types(),
            )
            .field(
                second,
                ApplicationFieldRef::<_, _, _, SecondNamedField, u64>::from_schema_types(),
            )
            .build()
    }
}

#[test]
fn duplicate_semantic_aspect_locus_denies_instead_of_overwriting_meaning() {
    let entity = AspectEntity::reference();
    let declaration = ApplicationSchemaDeclarationBuilder::for_schema()
        .entity(entity)
        .aspect(
            entity,
            ApplicationAspectRef::<_, _, FirstAspect>::from_schema_identifier("SameAspect"),
        )
        .aspect(
            entity,
            ApplicationAspectRef::<_, _, SecondAspect>::from_schema_identifier("SameAspect"),
        )
        .build()
        .unwrap();
    assert_package_contract_denial(
        declaration,
        WorthQueryPortablePackageValidationDenialKind::ApplicationContractDuplicateAspectLocus,
    );
}

#[test]
fn duplicate_semantic_field_locus_denies_instead_of_overwriting_shape() {
    let entity = FieldEntity::reference();
    let denial = ApplicationSchemaDeclarationBuilder::for_schema()
        .entity(entity)
        .aspect(entity, FieldAspect::reference())
        .field(
            entity,
            ApplicationFieldRef::<_, _, _, FirstField, u64>::from_schema_types(),
        )
        .field(
            entity,
            ApplicationFieldRef::<_, _, _, SecondField, String>::from_schema_types(),
        )
        .build()
        .unwrap_err();
    assert_eq!(
        denial,
        ApplicationSchemaDeclarationDenial::ConflictingFieldBinding
    );
}

#[test]
fn duplicate_identity_denies_across_distinct_entities() {
    let first = FirstIdentityEntity::reference();
    let second = SecondIdentityEntity::reference();
    let declaration = ApplicationSchemaDeclarationBuilder::for_schema()
        .entity(first)
        .entity(second)
        .aspect(
            first,
            ApplicationAspectRef::<_, _, FirstIdentityAspect>::from_schema_identifier(
                "FirstAspect",
            ),
        )
        .aspect(
            second,
            ApplicationAspectRef::<_, _, SecondIdentityAspect>::from_schema_identifier(
                "SecondAspect",
            ),
        )
        .build()
        .unwrap();
    assert_package_contract_denial(
        declaration,
        WorthQueryPortablePackageValidationDenialKind::ApplicationContractDuplicateAspectIdentity,
    );
}

#[test]
fn same_rendered_aspect_and_field_names_remain_distinct_across_entities() {
    let first = FirstNamedEntity::reference();
    let second = SecondNamedEntity::reference();
    let declaration = ApplicationSchemaDeclarationBuilder::for_schema()
        .entity(first)
        .entity(second)
        .aspect(
            first,
            ApplicationAspectRef::<_, _, FirstNamedAspect>::from_schema_identifier("SameAspect"),
        )
        .aspect(
            second,
            ApplicationAspectRef::<_, _, SecondNamedAspect>::from_schema_identifier("SameAspect"),
        )
        .field(
            first,
            ApplicationFieldRef::<_, _, _, FirstNamedField, u64>::from_schema_types(),
        )
        .field(
            second,
            ApplicationFieldRef::<_, _, _, SecondNamedField, u64>::from_schema_types(),
        )
        .build()
        .unwrap();
    let index = crate::facade::WorthQueryInstalledPackageIndex::build(
        crate::facade::WorthQueryInstallationRuntimeIdentity::fresh(),
        crate::facade::WorthQueryInstallationGeneration::initial(),
        [super::admitted_package(declaration.clone())],
    )
    .unwrap();
    let installed = index.bind_application_schema(declaration).unwrap();
    assert_eq!(
        installed
            .native_contracts()
            .aspect("FirstNamedEntity", "SameAspect")
            .unwrap()
            .contract()
            .identity(),
        AspectIdentity(41)
    );
    assert_eq!(
        installed
            .native_contracts()
            .aspect("SecondNamedEntity", "SameAspect")
            .unwrap()
            .contract()
            .identity(),
        AspectIdentity(42)
    );
}
