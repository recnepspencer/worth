use worth_foundational::facade::ScalarAspectType;

use super::canonical_identity::{canonical_identity, ApplicationSchemaCanonicalHeader};
use super::{ApplicationFieldPresence, ApplicationSchemaMember};

#[test]
fn required_and_optional_fields_have_distinct_schema_identity() {
    let required = field(ApplicationFieldPresence::Required);
    let optional = field(ApplicationFieldPresence::Optional);

    assert_ne!(identity(required), identity(optional));
}

fn identity(field: ApplicationSchemaMember) -> super::ApplicationSchemaIdentity {
    canonical_identity(
        ApplicationSchemaCanonicalHeader {
            owner: "worth.test",
            name: "FieldPresenceSchema",
            major: 1,
            minor: 0,
        },
        &[field],
        &[],
    )
}

fn field(presence: ApplicationFieldPresence) -> ApplicationSchemaMember {
    ApplicationSchemaMember::Field {
        entity: "Record".to_string(),
        aspect: "Facts".to_string(),
        field: "Value".to_string(),
        presence,
        scalar_family: ScalarAspectType::UInt64,
        value_type: std::any::type_name::<u64>().to_string(),
        unit: None,
        frame: None,
        writable: false,
        equality_queryable: true,
    }
}
