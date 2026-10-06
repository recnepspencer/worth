//! A unique field is looked up through its equality index, so a declaration
//! without one is refused for builders and archives alike.

use worth_foundational::facade::{AspectContractRevision, AspectIdentity, ScalarAspectType};

use super::member_closure::validate_member_closure;
use super::{
    ApplicationFieldPresence, ApplicationSchemaDeclarationDenial, ApplicationSchemaMember,
};

#[test]
fn unique_field_requires_an_equality_index() {
    assert_eq!(validate_member_closure(&members(true, true)), Ok(()));
    assert_eq!(
        validate_member_closure(&members(true, false)),
        Err(ApplicationSchemaDeclarationDenial::UniqueFieldWithoutEqualityIndex)
    );
    assert_eq!(validate_member_closure(&members(false, false)), Ok(()));
}

fn members(unique: bool, equality_queryable: bool) -> Vec<ApplicationSchemaMember> {
    vec![
        ApplicationSchemaMember::Entity {
            entity: "Record".to_owned(),
        },
        ApplicationSchemaMember::Aspect {
            entity: "Record".to_owned(),
            aspect: "Facts".to_owned(),
            identity: AspectIdentity(0x9161_3101),
            revision: AspectContractRevision(1),
        },
        ApplicationSchemaMember::Field {
            entity: "Record".to_owned(),
            aspect: "Facts".to_owned(),
            field: "Key".to_owned(),
            presence: ApplicationFieldPresence::Required,
            scalar_family: ScalarAspectType::UInt64,
            value_type: std::any::type_name::<u64>().to_owned(),
            unit: None,
            frame: None,
            writable: false,
            equality_queryable,
            unique,
        },
    ]
}
