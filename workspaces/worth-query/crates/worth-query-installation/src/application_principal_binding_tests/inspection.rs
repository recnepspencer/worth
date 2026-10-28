use super::*;
use worth_query_declaration::facade::{
    application_query::*,
    application_schema::{
        ApplicationSchema, ApplicationSchemaDeclaration, ApplicationSchemaDeclarationBuilder,
    },
};

fn declaration(inspect: bool) -> ApplicationSchemaDeclaration<IdentitySchema> {
    let schema = schema_members::declare(ApplicationSchemaDeclarationBuilder::for_schema());
    let schema = if inspect {
        schema.application_query_inspection(
            ApplicationQueryInspectionDefinition::<IdentitySchema, PrincipalQueryBinding>::new(
                ApplicationQueryDisclosureContract::public(),
            )
            .unwrap(),
        )
    } else {
        schema
    };
    schema.build().unwrap()
}

#[test]
fn typed_inspection_meaning_refuses_inert_installation_and_plain_binding_still_installs() {
    use crate::facade::WorthQueryInstalledApplicationSchemaDenialKind as Kind;
    for inspect in [false, true] {
        let package = WorthQueryPortableDomainPackage::new(WorthQueryPortableDomainIdentity::new(
            "identity_installation_test",
            1,
            0,
        ))
        .application_schema(declaration(inspect))
        .validate()
        .unwrap();
        let admitted = WorthQueryInstallationAdmissionProfile::new("support", "configuration")
            .admit(package)
            .unwrap();
        let index = WorthQueryInstalledPackageIndex::build(
            WorthQueryInstallationRuntimeIdentity::fresh(),
            WorthQueryInstallationGeneration::initial(),
            [admitted],
        )
        .unwrap();
        match index.bind_application_schema(declaration(inspect)) {
            Ok(schema) => {
                assert!(
                    !inspect,
                    "metadata meaning cannot silently install without its issuer"
                );
                schema
                    .installed_query_binding::<PrincipalQueryBinding>()
                    .unwrap();
            }
            Err(denial) => {
                assert!(inspect);
                assert_eq!(denial.kind(), Kind::InspectionUnavailable);
                assert_eq!(
                    denial.subject(),
                    "worth.query.tests.principal-query-binding.v1"
                );
            }
        }
    }
}

#[test]
fn typed_inspection_rules_are_canonical_for_both_authoring_orders() {
    use worth_query_declaration::facade::{
        application_schema::{ApplicationEncodedScalarValue, BoolApplicationValueBinding},
        portable_identity::WorthQueryPortableTypeIdentity,
    };
    let contract = |reverse: bool| {
        let mut facets = ApplicationQueryInspectionMetadataFacet::ALL.to_vec();
        if reverse {
            facets.reverse();
        }
        let metadata = ApplicationQueryDisclosureContract::from_untrusted_parts(
            WorthQueryPortableApplicationQueryDisclosureParts {
                posture: ApplicationQueryDisclosurePosture::Governed,
                classification: "inspection".into(),
                capability_name: Some("Inspect".into()),
                capability_type: Some(WorthQueryPortableTypeIdentity::declared(
                    "worth.test.inspect.v1",
                )),
                rules: vec![],
            },
        );
        facets.into_iter().fold(metadata, |contract, facet| {
            contract.disclose_inspection_metadata_by(
                ApplicationQueryInspectionMetadataRef::<IdentitySchema, PrincipalQueryBinding>::new(
                    facet,
                ),
                ApplicationEncodedScalarValue::<BoolApplicationValueBinding>::try_new(true)
                    .unwrap(),
                ApplicationQueryInfluenceContract::permit([facet.influence()]),
            )
        })
    };
    let forward =
        ApplicationQueryInspectionDefinition::<IdentitySchema, PrincipalQueryBinding>::new(
            contract(false),
        )
        .unwrap()
        .into_erased();
    let reverse =
        ApplicationQueryInspectionDefinition::<IdentitySchema, PrincipalQueryBinding>::new(
            contract(true),
        )
        .unwrap()
        .into_erased();
    assert_eq!(forward.canonical_basis(), reverse.canonical_basis());
}
