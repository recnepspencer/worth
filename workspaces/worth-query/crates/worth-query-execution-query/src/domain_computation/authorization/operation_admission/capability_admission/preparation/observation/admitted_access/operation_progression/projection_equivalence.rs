use worth_query_declaration::facade::application_capability::ApplicationCapabilityRequestProjection;

pub fn same_projection<Schema, Scope, Context>(
    expected: &ApplicationCapabilityRequestProjection<Schema, Scope, Context>,
    actual: &ApplicationCapabilityRequestProjection<Schema, Scope, Context>,
) -> bool {
    same_resource(expected, actual)
        && expected.elevation_selector() == actual.elevation_selector()
        && expected.action() == actual.action()
        && expected.purpose() == actual.purpose()
        && same_related(expected, actual)
        && expected.field_value() == actual.field_value()
        && expected.magnitude_value() == actual.magnitude_value()
        && expected.cardinality_value() == actual.cardinality_value()
        && expected.context_value().context() == actual.context_value().context()
        && expected.context_value().context_type() == actual.context_value().context_type()
        && expected.context_value().entities() == actual.context_value().entities()
}

fn same_resource<Schema, Scope, Context>(
    expected: &ApplicationCapabilityRequestProjection<Schema, Scope, Context>,
    actual: &ApplicationCapabilityRequestProjection<Schema, Scope, Context>,
) -> bool {
    expected.resource().entity() == actual.resource().entity()
        && expected.resource().aspect() == actual.resource().aspect()
        && expected.resource().field() == actual.resource().field()
        && expected.resource().scalar_family() == actual.resource().scalar_family()
        && expected.resource().value_type() == actual.resource().value_type()
        && expected.resource().value() == actual.resource().value()
}

fn same_related<Schema, Scope, Context>(
    expected: &ApplicationCapabilityRequestProjection<Schema, Scope, Context>,
    actual: &ApplicationCapabilityRequestProjection<Schema, Scope, Context>,
) -> bool {
    match (expected.related(), actual.related()) {
        (None, None) => true,
        (Some(expected), Some(actual)) => {
            expected.relation() == actual.relation() && expected.selector() == actual.selector()
        }
        _ => false,
    }
}
