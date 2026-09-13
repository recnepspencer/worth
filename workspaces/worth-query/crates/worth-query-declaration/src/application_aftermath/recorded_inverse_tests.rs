use crate::application_schema::ApplicationFieldRef;

use super::{DeclaredPreImageDemand, DeclaredPreImageDemandDenial, DeclaredPreImageLocus};

struct Schema;
struct FirstEntity;
struct SecondEntity;
struct Aspect;
struct FirstField;
struct SecondField;

macro_rules! declare_u64_field {
    ($($field:ty),+ $(,)?) => {$ (
        impl crate::application_schema::DeclaredApplicationFieldValue for $field {
            type Value = u64;
            type Binding = crate::application_schema::U64ApplicationValueBinding;
            const PRESENCE: crate::application_schema::ApplicationFieldPresence =
                crate::application_schema::ApplicationFieldPresence::Required;
        }
    )+};
}

declare_u64_field!(FirstField, SecondField);

#[test]
fn duplicate_exact_locus_is_denied() {
    let locus = locus::<FirstEntity, FirstField>("FirstEntity", "Aspect", "FirstField");
    assert_eq!(
        DeclaredPreImageDemand::new([locus.clone(), locus], 64),
        Err(DeclaredPreImageDemandDenial::DuplicateLocus)
    );
}

#[test]
fn multiple_entity_roles_are_denied_before_installation() {
    let first = locus::<FirstEntity, FirstField>("FirstEntity", "Aspect", "FirstField");
    let second = locus::<SecondEntity, SecondField>("SecondEntity", "Aspect", "SecondField");
    assert_eq!(
        DeclaredPreImageDemand::new([first, second], 64),
        Err(DeclaredPreImageDemandDenial::MultipleEntityRoles)
    );
}

#[test]
fn multiple_exact_fields_on_one_entity_role_are_admissible() {
    let first = locus::<FirstEntity, FirstField>("FirstEntity", "FirstAspect", "FirstField");
    let second = locus::<FirstEntity, SecondField>("FirstEntity", "SecondAspect", "SecondField");
    let demand = DeclaredPreImageDemand::new([first, second], 64).unwrap();
    assert_eq!(demand.loci().len(), 2);
}

fn locus<Entity, Field: crate::application_schema::DeclaredApplicationFieldValue<Value = u64>>(
    entity: &'static str,
    aspect: &'static str,
    field: &'static str,
) -> DeclaredPreImageLocus<Schema> {
    DeclaredPreImageLocus::from_field(
        ApplicationFieldRef::<Schema, Entity, Aspect, Field, u64>::from_schema_identifiers(
            entity, aspect, field,
        ),
    )
}
