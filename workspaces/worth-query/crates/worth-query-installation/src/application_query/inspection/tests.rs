use super::*;
use crate::application_principal_binding_tests::{IdentitySchema, PrincipalQueryBinding};
use crate::facade::{
    WorthQueryInstallationAdmissionProfile, WorthQueryInstallationGeneration,
    WorthQueryInstallationRuntimeIdentity, WorthQueryInstalledPackageIndex,
    WorthQueryPortableDomainIdentity, WorthQueryPortableDomainPackage,
};
use worth_query_declaration::facade::application_query::{
    ApplicationQueryBindingReference, ApplicationQueryDisclosureContract,
    ApplicationQueryInspectionDefinition, ErasedApplicationQueryInspectionDefinition,
};
use worth_query_declaration::facade::portable_identity::WorthQueryPortableTypeIdentity;

#[test]
fn inspection_resolves_every_definition_against_actual_compiled_binding_before_installation() {
    let package = WorthQueryPortableDomainPackage::new(WorthQueryPortableDomainIdentity::new(
        "identity_installation_test",
        1,
        0,
    ))
    .application_schema(IdentitySchema::declaration().unwrap())
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
    let schema = index
        .bind_application_schema(IdentitySchema::declaration().unwrap())
        .unwrap();
    let definition =
        ApplicationQueryInspectionDefinition::<IdentitySchema, PrincipalQueryBinding>::new(
            ApplicationQueryDisclosureContract::public(),
        )
        .unwrap()
        .into_erased();
    let reference = &definition.parts().binding;
    let name = reference.parts().binding.as_str();
    let bindings = BTreeMap::from([(
        name.to_owned(),
        schema.query_catalog.get_binding(name).unwrap(),
    )]);
    let valid = ApplicationSchemaMember::ApplicationQueryInspection {
        definition: definition.clone(),
    };
    validate_definitions(&[valid.clone()], &bindings).unwrap();
    // Compile the actual installed binding's independent cold obligation. Its
    // ordinary query graph remains a different subject and grants no metadata.
    let compiled = bindings.get(name).unwrap();
    let meaning = CompiledInspectionMeaning::compile(&schema, compiled, &definition).unwrap();
    use crate::graph_obligation::{
        WorthQueryInstalledGraphAuthorizationRequirement as Requirement,
        WorthQueryInstalledGraphObligationSubjectKind as Subject,
    };
    assert_eq!(
        meaning.obligations.subject_kind(),
        Subject::ApplicationInspection
    );
    assert!(meaning.obligations.application_query_identity().is_none());
    assert!(meaning.obligations.rows().iter().all(|row| matches!(
        row.authorization_requirement(),
        Some(Requirement::Principal | Requirement::Abilities(_))
    )));
    assert_eq!(
        compiled.query().graph_obligations().subject_kind(),
        Subject::ApplicationQuery
    );
    let other = ApplicationQueryInspectionDefinition::<
        IdentitySchema,
        crate::application_principal_binding_tests::ResultOnlyQueryBinding,
    >::new(ApplicationQueryDisclosureContract::public())
    .unwrap()
    .into_erased();
    let other_name = other.parts().binding.parts().binding.as_str();
    let other_binding = schema.query_catalog.get_binding(other_name).unwrap();
    assert_eq!(compiled.query_identity(), other_binding.query_identity());
    let other_meaning =
        CompiledInspectionMeaning::compile(&schema, &other_binding, &other).unwrap();
    assert_ne!(
        meaning.obligations.identity(),
        other_meaning.obligations.identity()
    );
    let missing_capability = missing_capability_definition();
    assert!(matches!(
        CompiledInspectionMeaning::compile(&schema, compiled, &missing_capability),
        Err(cause) if cause.kind() == Kind::InvalidInspectionDefinition
    ));
    // The installed descriptor is genuine; only the untrusted descriptive input
    // changes. Matching the binding's display name would accept this counterfeit.
    let mut parts = definition.parts().clone();
    let mut changed = parts.binding.parts().clone();
    changed.input = WorthQueryPortableTypeIdentity::declared("worth.query.foreign-input.v1");
    parts.binding = ApplicationQueryBindingReference::from_untrusted_parts(changed);
    let foreign = ApplicationSchemaMember::ApplicationQueryInspection {
        definition: ErasedApplicationQueryInspectionDefinition::from_untrusted_parts(parts),
    };
    assert_eq!(
        validate_definitions(&[foreign.clone()], &bindings)
            .unwrap_err()
            .kind(),
        Kind::InspectionBindingMeaningChanged
    );
    // Distinct actual bindings reach descriptor validation without the earlier
    // duplicate-definition gate masking the later foreign input meaning.
    let precedence_bindings = BTreeMap::from([
        (name.to_owned(), Arc::clone(compiled)),
        (other_name.to_owned(), Arc::clone(&other_binding)),
    ]);
    let mut foreign_other_parts = other.parts().clone();
    let mut foreign_other_binding = foreign_other_parts.binding.parts().clone();
    foreign_other_binding.input =
        WorthQueryPortableTypeIdentity::declared("worth.query.foreign-input.v1");
    foreign_other_parts.binding =
        ApplicationQueryBindingReference::from_untrusted_parts(foreign_other_binding);
    let foreign_other = ApplicationSchemaMember::ApplicationQueryInspection {
        definition: ErasedApplicationQueryInspectionDefinition::from_untrusted_parts(
            foreign_other_parts,
        ),
    };
    assert_eq!(
        validate_all(
            &[
                ApplicationSchemaMember::ApplicationQueryInspection {
                    definition: missing_capability
                },
                foreign_other,
            ],
            &precedence_bindings,
        )
        .err()
        .unwrap()
        .kind(),
        Kind::InspectionBindingMeaningChanged
    );
    let mut parts = definition.parts().clone();
    let mut missing = parts.binding.parts().clone();
    missing.binding =
        WorthQueryPortableTypeIdentity::declared("worth.query.uninstalled-binding.v1");
    parts.binding = ApplicationQueryBindingReference::from_untrusted_parts(missing);
    let missing = ApplicationSchemaMember::ApplicationQueryInspection {
        definition: ErasedApplicationQueryInspectionDefinition::from_untrusted_parts(parts),
    };
    assert_eq!(
        validate_definitions(&[valid, missing], &bindings)
            .unwrap_err()
            .kind(),
        Kind::InspectionBindingMeaningChanged
    );
}

fn missing_capability_definition() -> ErasedApplicationQueryInspectionDefinition {
    use worth_query_declaration::facade::{
        application_query::*,
        application_schema::{ApplicationEncodedScalarValue, BoolApplicationValueBinding},
    };
    let contract = ApplicationQueryDisclosureContract::from_untrusted_parts(
        WorthQueryPortableApplicationQueryDisclosureParts {
            posture: ApplicationQueryDisclosurePosture::Governed,
            classification: "inspection".into(),
            capability_name: Some("MissingInspect".into()),
            capability_type: Some(WorthQueryPortableTypeIdentity::declared(
                "worth.test.missing-inspect.v1",
            )),
            rules: vec![],
        },
    );
    let contract = ApplicationQueryInspectionMetadataFacet::ALL
        .into_iter()
        .fold(contract, |contract, facet| {
            contract.disclose_inspection_metadata_by(
                ApplicationQueryInspectionMetadataRef::<IdentitySchema, PrincipalQueryBinding>::new(
                    facet,
                ),
                ApplicationEncodedScalarValue::<BoolApplicationValueBinding>::try_new(true)
                    .unwrap(),
                ApplicationQueryInfluenceContract::permit([facet.influence()]),
            )
        });
    ApplicationQueryInspectionDefinition::<IdentitySchema, PrincipalQueryBinding>::new(contract)
        .unwrap()
        .into_erased()
}
