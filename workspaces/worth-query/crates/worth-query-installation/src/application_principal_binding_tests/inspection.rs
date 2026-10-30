use super::*;
use worth_query_declaration::facade::{
    application_query::*,
    application_schema::{ApplicationSchemaDeclaration, ApplicationSchemaDeclarationBuilder},
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
fn typed_inspection_installs_its_actual_independent_metadata_obligation() {
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
        let schema = index.bind_application_schema(declaration(inspect)).unwrap();
        let binding = schema
            .installed_query_binding::<PrincipalQueryBinding>()
            .unwrap();
        let observation = binding.query().binding_observation().unwrap();
        let contract = observation.inspection_contract();
        assert_eq!(contract.is_some(), inspect);
        if let Some(contract) = contract {
            assert_eq!(contract.binding_observation(), observation);
            assert_eq!(contract.graph_obligations().subject_kind(),
                crate::graph_obligation::WorthQueryInstalledGraphObligationSubjectKind::ApplicationInspection);
            assert_ne!(
                contract.graph_obligations().identity(),
                binding.query().graph_obligations().identity()
            );
            assert_eq!(contract.metadata_slots().len(), 6);
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
