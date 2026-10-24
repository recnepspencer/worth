//! Untrusted field manifests must match the actual created role and value meaning.
use super::*;
use worth_query_declaration::facade::application_operation::{
    WorthQueryApplicationDeclaredOutputRole, WorthQueryApplicationOutputRole,
    WorthQueryCreateOutput, WorthQueryExactlyOneOutput,
};

struct CreatedOutputs;
struct Created;
impl ApplicationMutationOutputContract<MutationSchema> for CreatedOutputs {
    const ROLES: &'static [ApplicationMutationOutputRoleDescriptor] = &[Created::DESCRIPTOR];
}
impl WorthQueryApplicationOutputRole for Created {
    type Schema = MutationSchema;
    type Contract = CreatedOutputs;
    type Entity = FixtureEntity<MutationSchema>;
    type Action = WorthQueryCreateOutput;
    type Cardinality = WorthQueryExactlyOneOutput;
    const NAME: &'static str = "created-account";
}
mutation_binding!(
    CreatedBinding,
    "fixture.created.v1",
    TestOperation<MutationSchema>,
    TestInput,
    TestInputBinding,
    MutationDecision,
    MutationDenialBinding,
    "fixture.created-handler.v1",
    FixturePrincipalIdentityField<MutationSchema>,
    test_principal_binding::<MutationSchema>(),
    CreatedOutputs
);

#[test]
fn source_output_manifest_resolves_actual_role_and_complete_field_meaning() {
    let source = CreatedBinding::descriptor();
    let inventory = BTreeMap::from([(source.identity().to_owned(), &source)]);
    let queries = [disclosure::HistoryBinding::descriptor()];
    let field = ApplicationFieldRef::<
        MutationSchema,
        FixtureEntity<MutationSchema>,
        FixtureIdentityAspect<MutationSchema>,
        FixturePrincipalIdentityField<MutationSchema>,
        u64,
        ReadOnly,
        EqualityPredicate,
        NoApplicationUnit,
    >::from_schema_types();
    let witness = ApplicationCorrectionOutput::<CreatedBinding, Created>::field(field);
    let declared = witness.declaration().parts().clone();
    for axis in [
        "valid",
        "codec",
        "unit",
        "binding-unit",
        "frame",
        "source",
        "role",
        "entity",
        "presence",
        "scalar",
        "duplicate",
    ] {
        let mut altered = declared.clone();
        match axis {
            "valid" | "duplicate" => {}
            "codec" => altered.value_binding = identity("different-codec"),
            "unit" => altered.unit = Some("different-unit".to_owned()),
            "binding-unit" => altered.binding_unit = Some("different-unit".to_owned()),
            "frame" => altered.frame = Some("different-frame".to_owned()),
            "source" => altered.source_binding = identity("different-source"),
            "role" => altered.role = "different-role".to_owned(),
            "entity" => altered.entity = "different-entity".to_owned(),
            "presence" => altered.presence = ApplicationFieldPresence::Optional,
            "scalar" => altered.scalar_family = worth_foundational::facade::ScalarAspectType::Bool,
            _ => unreachable!(),
        }
        let mut association = parts(&source);
        association.source_output_fields =
            vec![DeclaredCorrectionOutputField::from_untrusted_parts(altered)];
        if axis == "duplicate" {
            association
                .source_output_fields
                .push(witness.declaration().clone());
        }
        let mut actual = members(&source, association);
        actual.push(witness.declaration().projected_value_member());
        let result = resolve_correction_binding(&actual, &source, &inventory, &queries);
        if axis == "valid" {
            assert!(result.unwrap().is_some());
        } else {
            assert_eq!(result.unwrap_err().kind(), crate::application_operation::WorthQueryApplicationOperationInstallationDenialKind::AftermathInstallationDenied, "{axis}");
        }
    }
}
