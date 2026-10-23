//! Installation resolves lifecycle families against actual field and output owners.
use super::*;
use worth_query_declaration::facade::application_operation::{
    ApplicationMutationOutputPosture as Posture, WorthQueryApplicationDeclaredOutputRole,
    WorthQueryApplicationOutputRole, WorthQueryCreateOutput, WorthQueryExactlyOneOutput,
    WorthQueryRetireOutput,
};
use worth_query_declaration::facade::application_schema::ApplicationOperationProgramTarget as Target;

struct CreatedOutputs;
struct RetiredOutputs;
struct Created;
struct Retired;
macro_rules! output {
    ($Outputs:ident, $Role:ident, $Action:ty, $name:literal) => {
        impl ApplicationMutationOutputContract<MutationSchema> for $Outputs {
            const ROLES: &'static [ApplicationMutationOutputRoleDescriptor] = &[$Role::DESCRIPTOR];
        }
        impl WorthQueryApplicationOutputRole for $Role {
            type Schema = MutationSchema;
            type Contract = $Outputs;
            type Entity = FixtureEntity<MutationSchema>;
            type Action = $Action;
            type Cardinality = WorthQueryExactlyOneOutput;
            const NAME: &'static str = $name;
        }
    };
}
output!(CreatedOutputs, Created, WorthQueryCreateOutput, "created");
output!(RetiredOutputs, Retired, WorthQueryRetireOutput, "retired");
mutation_binding!(
    CreatedBinding,
    "fixture.authored.create.v1",
    TestOperation<MutationSchema>,
    TestInput,
    TestInputBinding,
    MutationDecision,
    MutationDenialBinding,
    "fixture.authored.create-handler.v1",
    FixturePrincipalIdentityField<MutationSchema>,
    test_principal_binding::<MutationSchema>(),
    CreatedOutputs
);
mutation_binding!(
    RetiredBinding,
    "fixture.authored.retire.v1",
    TestOperation<MutationSchema>,
    TestInput,
    TestInputBinding,
    MutationDecision,
    MutationDenialBinding,
    "fixture.authored.retire-handler.v1",
    FixturePrincipalIdentityField<MutationSchema>,
    test_principal_binding::<MutationSchema>(),
    RetiredOutputs
);

#[test]
fn authored_family_rejects_missing_role_or_changed_logical_key_meaning() {
    let source = CreatedBinding::descriptor();
    let inverse = RetiredBinding::descriptor();
    let inventory = BTreeMap::from([
        (source.identity().to_owned(), &source),
        (inverse.identity().to_owned(), &inverse),
    ]);
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
    let key = ApplicationCorrectionOutput::<CreatedBinding, Created>::field(field)
        .declaration()
        .projected_value_member();
    for axis in [
        "valid",
        "missing-family",
        "wrong-role",
        "wrong-posture",
        "wrong-key-codec",
        "wrong-binding-unit",
        "wrong-key-presence",
        "missing-create",
        "conflicting-family",
    ] {
        let mut identity_parts = DeclaredAuthoredCorrectionParts {
            family: identity("authored-fixture.v1"),
            logical_key: Box::new(key.clone()),
            binding_unit: None,
            source_role: "created".to_owned(),
            inverse_role: "retired".to_owned(),
            source_posture: Posture::Create,
        };
        match axis {
            "wrong-role" => identity_parts.inverse_role = "foreign".to_owned(),
            "wrong-posture" => identity_parts.source_posture = Posture::Preserve,
            "wrong-binding-unit" => identity_parts.binding_unit = Some("foreign-unit".to_owned()),
            "wrong-key-codec" => {
                if let ApplicationSchemaMember::Field { value_type, .. } =
                    identity_parts.logical_key.as_mut()
                {
                    *value_type = "foreign-codec".to_owned();
                }
            }
            "wrong-key-presence" => {
                if let ApplicationSchemaMember::Field { presence, .. } =
                    identity_parts.logical_key.as_mut()
                {
                    *presence = ApplicationFieldPresence::Optional;
                }
            }
            _ => (),
        }
        let mut association = parts(&source);
        association.inverse = inverse.description().clone();
        association.inverse_handler = inverse.handler().identity().to_owned();
        association.authored_identity = (axis != "missing-family")
            .then(|| DeclaredAuthoredCorrection::from_untrusted_parts(identity_parts.clone()));
        let contract = |association| {
            PortableApplicationAftermathContract::from_untrusted_fields(
                DeclaredCorrectionAuthority::RuntimeAlone,
                Some(PortableCorrectionMechanism::RecordedInverse(
                    PortableRecordedInverse::from_untrusted_fields(
                        inverse.operation_name().to_owned(),
                        DeclaredLoweringCorrespondenceRef::new("fixture.authored.inverse.v1")
                            .unwrap(),
                        DeclaredAftermathPostcondition::ExactPriorTruth,
                        PortablePreImageDemand::from_untrusted_fields(vec![], 0),
                    ),
                )),
                None,
            )
            .with_untrusted_correction_binding(
                DeclaredCorrectionBinding::from_untrusted_parts(association),
            )
        };
        let mut actual = vec![
            key.clone(),
            ApplicationSchemaMember::OperationAftermath {
                operation: source.operation_name().to_owned(),
                contract: contract(association),
            },
        ];
        let ApplicationSchemaMember::Field { entity, .. } = &key else {
            unreachable!()
        };
        if axis != "missing-create" {
            actual.push(ApplicationSchemaMember::OperationProgram {
                operation: source.operation_name().to_owned(),
                target: Target::Create {
                    entity: entity.clone(),
                },
            });
        }
        actual.push(ApplicationSchemaMember::OperationProgram {
            operation: inverse.operation_name().to_owned(),
            target: Target::Delete {
                entity: entity.clone(),
            },
        });
        if axis == "conflicting-family" {
            let mut association = parts(&source);
            association.inverse = inverse.description().clone();
            association.inverse_handler = inverse.handler().identity().to_owned();
            let mut conflicting = identity_parts;
            if let ApplicationSchemaMember::Field { value_type, .. } =
                conflicting.logical_key.as_mut()
            {
                *value_type = "different-family-codec".to_owned();
            }
            association.authored_identity = Some(DeclaredAuthoredCorrection::from_untrusted_parts(
                conflicting,
            ));
            actual.push(ApplicationSchemaMember::OperationAftermath {
                operation: "another-operation".to_owned(),
                contract: contract(association),
            });
        }
        let result = resolve_correction_binding(&actual, &source, &inventory, &queries);
        if axis == "valid" {
            assert!(result.unwrap().is_some());
        } else {
            assert_eq!(result.unwrap_err().kind(), crate::application_operation::WorthQueryApplicationOperationInstallationDenialKind::AftermathInstallationDenied, "{axis}");
        }
    }
}
