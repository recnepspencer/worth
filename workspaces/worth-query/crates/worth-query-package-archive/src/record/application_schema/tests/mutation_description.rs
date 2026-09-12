use super::{decode_member, encode_member};
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
        let description = ApplicationMutationDescription::from_untrusted_parts(
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
                    },
                )
                .collect(),
            },
        );
        let member = ApplicationSchemaMember::ApplicationMutation { description };
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
    }
}

fn identity(name: &str) -> WorthQueryPortableTypeIdentity {
    WorthQueryPortableTypeIdentity::from_untrusted(name.to_owned())
}
