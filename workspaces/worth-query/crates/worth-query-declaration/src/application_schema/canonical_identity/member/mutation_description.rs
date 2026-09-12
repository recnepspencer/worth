use crate::application_operation::{
    ApplicationMutationDescription, ApplicationMutationOutputPosture,
    ApplicationMutationScopeResolutionMode,
};
use crate::application_schema::canonical_basis::ApplicationSchemaCanonicalBasis;

pub(super) fn append(
    basis: &mut ApplicationSchemaCanonicalBasis,
    prefix: &str,
    description: &ApplicationMutationDescription,
) {
    basis.text(format!("{prefix}.kind"), "application-mutation");
    for (field, value) in [
        ("binding-identity", description.binding_identity().as_str()),
        ("operation", description.operation()),
        ("input-identity", description.input_identity().as_str()),
        ("result-identity", description.result_identity().as_str()),
        ("denial-identity", description.denial_identity().as_str()),
        ("scope.entity", &description.scope().entity),
        ("scope.aspect", &description.scope().aspect),
        ("scope.field", &description.scope().field),
    ] {
        basis.text(format!("{prefix}.{field}"), value);
    }
    basis.text(
        format!("{prefix}.scope.resolution"),
        match description.scope().resolution {
            ApplicationMutationScopeResolutionMode::InputField => "input-field",
            ApplicationMutationScopeResolutionMode::PrincipalIdentity => "principal-identity",
        },
    );
    basis.usize(
        format!("{prefix}.output-role-count"),
        description.output_roles().len(),
    );
    for (index, role) in description.output_roles().iter().enumerate() {
        let prefix = format!("{prefix}.output-role[{index}]");
        basis.text(format!("{prefix}.name"), &role.name);
        basis.text(format!("{prefix}.entity"), &role.entity);
        basis.text(
            format!("{prefix}.posture"),
            match role.posture {
                ApplicationMutationOutputPosture::Preserve => "preserve",
                ApplicationMutationOutputPosture::Create => "create",
                ApplicationMutationOutputPosture::Retire => "retire",
            },
        );
    }
}
