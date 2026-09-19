use worth_query_declaration::facade::application_schema::ApplicationSchemaMember;

pub(super) fn operation_capability_count(
    members: &[ApplicationSchemaMember],
    operation: &str,
    input_type: &str,
) -> usize {
    members
        .iter()
        .filter(|member| match member {
            ApplicationSchemaMember::ApplicationCapability { contract } => {
                contract.operation() == operation && contract.input_type() == input_type
            }
            _ => false,
        })
        .count()
}

pub(super) fn progression_support_fact_count(
    members: &[ApplicationSchemaMember],
    operation: &str,
    input_type: &str,
) -> usize {
    usize::from(members.iter().any(|member| {
        match member {
            ApplicationSchemaMember::ApplicationCapability { contract } => {
                contract.elevation().definition().is_some_and(|definition| {
                    [
                        definition.lifecycle().request(),
                        definition.lifecycle().approve(),
                    ]
                    .into_iter()
                    .any(|transition| {
                        transition.operation().operation() == operation
                            && transition.operation().input_type() == input_type
                    })
                }) || contract
                    .delegation()
                    .activation()
                    .is_some_and(|activation| {
                        activation.operation().operation() == operation
                            && activation.operation().input_type() == input_type
                    })
            }
            _ => false,
        }
    }))
}
