use super::{decode_member, encode_member, try_decode_member};
use crate::facade::WorthQueryPackageArchiveDenialKind;
use worth_query_declaration::facade::{
    application_operation::*, application_schema::ApplicationSchemaMember,
    portable_identity::WorthQueryPortableTypeIdentity,
};

#[test]
fn mutation_archive_retains_typed_failure_scope_and_output_postures_without_native_types() {
    for resolution in [
        ApplicationMutationScopeResolutionMode::InputField,
        ApplicationMutationScopeResolutionMode::PrincipalIdentity,
    ] {
        let member = mutation_member(
            resolution,
            ApplicationMutationOutputRoleCardinality::AtMostOne,
        );
        let encoded = encode_member(&member);
        let decoded = decode_member(&encoded);
        assert_eq!(decoded, member);
        assert_eq!(encode_member(&decoded), encoded);
        let ApplicationSchemaMember::ApplicationMutation { description } = decoded else {
            unreachable!()
        };
        assert_eq!(
            description.denial_identity().as_str(),
            "worth.archive.denial.v1"
        );
        assert_eq!(description.scope().resolution, resolution);
        assert_eq!(
            description
                .output_roles()
                .iter()
                .map(|role| role.cardinality)
                .collect::<Vec<_>>(),
            [
                ApplicationMutationOutputRoleCardinality::ExactlyOne,
                ApplicationMutationOutputRoleCardinality::AtMostOne,
                ApplicationMutationOutputRoleCardinality::ExactlyOne,
            ]
        );
    }
}

#[test]
fn mutation_archive_encodes_each_output_role_cardinality_distinctly() {
    let resolution = ApplicationMutationScopeResolutionMode::InputField;
    let exact = mutation_member(
        resolution,
        ApplicationMutationOutputRoleCardinality::ExactlyOne,
    );
    let optional = mutation_member(
        resolution,
        ApplicationMutationOutputRoleCardinality::AtMostOne,
    );

    assert_ne!(encode_member(&exact), encode_member(&optional));
    assert_eq!(decode_member(&encode_member(&exact)), exact);
    assert_eq!(decode_member(&encode_member(&optional)), optional);
}

#[test]
fn mutation_archive_refuses_an_unknown_output_role_cardinality() {
    let resolution = ApplicationMutationScopeResolutionMode::InputField;
    let exact = encode_member(&mutation_member(
        resolution,
        ApplicationMutationOutputRoleCardinality::ExactlyOne,
    ));
    let mut unknown = encode_member(&mutation_member(
        resolution,
        ApplicationMutationOutputRoleCardinality::AtMostOne,
    ));
    let tag = exact
        .iter()
        .zip(&unknown)
        .position(|(exact, optional)| exact != optional)
        .expect("the cardinalities encode differently");
    assert_eq!((exact[tag], unknown[tag]), (1, 2));
    unknown[tag] = 3;

    assert_eq!(
        try_decode_member(&unknown).unwrap_err().kind(),
        WorthQueryPackageArchiveDenialKind::UnsupportedRecordVariant
    );
}

/// A mutation whose middle output role carries `middle` and whose other roles
/// are exactly-one.
fn mutation_member(
    resolution: ApplicationMutationScopeResolutionMode,
    middle: ApplicationMutationOutputRoleCardinality,
) -> ApplicationSchemaMember {
    ApplicationSchemaMember::ApplicationMutation {
        description: ApplicationMutationDescription::from_untrusted_parts(
            ApplicationMutationDescriptionParts {
                binding_identity: identity("worth.archive.mutation.v1"),
                operation: "Publish".to_owned(),
                input_identity: identity("worth.archive.input.v1"),
                result_identity: identity("worth.archive.result.v1"),
                denial_identity: identity("worth.archive.denial.v1"),
                scope: ApplicationMutationScopeDescription {
                    entity: "Body".to_owned(),
                    aspect: "Identity".to_owned(),
                    field: "Key".to_owned(),
                    resolution,
                },
                output_roles: [
                    ApplicationMutationOutputPosture::Preserve,
                    ApplicationMutationOutputPosture::Create,
                    ApplicationMutationOutputPosture::Retire,
                ]
                .into_iter()
                .enumerate()
                .map(
                    |(index, posture)| ApplicationMutationOutputRoleDescription {
                        name: format!("role-{index}"),
                        entity: "Body".to_owned(),
                        posture,
                        cardinality: if index == 1 {
                            middle
                        } else {
                            ApplicationMutationOutputRoleCardinality::ExactlyOne
                        },
                    },
                )
                .collect(),
                output_role_families: vec![ApplicationMutationOutputRoleFamilyDescription {
                    prefix: "face.".to_owned(),
                    entity: "Body".to_owned(),
                    postures: ApplicationMutationOutputPostureSet::ALL,
                    minimum: 2,
                }],
            },
        ),
    }
}

fn identity(name: &str) -> WorthQueryPortableTypeIdentity {
    WorthQueryPortableTypeIdentity::from_untrusted(name.to_owned())
}
