//! Resource accounting for source-bearing native operation bodies.

use super::*;
use worth_foundational::facade::ScalarAspectType;
use worth_query_declaration::facade::{
    application_schema::{
        ApplicationFieldBindingLocus, ApplicationOperationBodyProgram,
        ApplicationOperationBodyScopeField,
    },
    portable_identity::WorthQueryPortableTypeIdentity,
};

#[test]
fn clear_optional_portable_operation_reconstruction_accounts_without_input() {
    let record = WorthQueryPortableApplicationOperationContractRecord::new(
        "Schema".to_owned(),
        "Clear".to_owned(),
        WorthQueryPortableTypeIdentity::from_untrusted("Input".to_owned()),
        vec![],
        vec![],
        vec![],
        None,
        None,
    )
    .with_body(Some(
        ApplicationOperationBodyProgram::from_untrusted_scope_fields(vec![
            ApplicationOperationBodyScopeField::ClearOptional {
                field: ApplicationFieldBindingLocus::new("Item", "Draft", "Note"),
                binding: WorthQueryPortableTypeIdentity::from_untrusted(
                    "worth.rust.string".to_owned(),
                ),
                scalar_family: ScalarAspectType::String,
            },
        ]),
    ));
    let work = record.reconstruction_work();
    assert!(work.logical_bytes > 0);
    assert!(work.nested_entries >= 7);
}

#[test]
fn principal_initializer_reconstruction_accounts_for_every_source_component() {
    use worth_query_declaration::facade::application_schema::{
        ApplicationOperationBodyCreatedField as Field, ApplicationOperationBodyCreation,
    };
    let value = worth_foundational::facade::AspectValue::UInt64(1);
    let target = ApplicationFieldBindingLocus::new("Audit", "Facts", "Actor");
    let binding = WorthQueryPortableTypeIdentity::from_untrusted("worth.rust.u64".to_owned());
    let record = |step| {
        WorthQueryPortableApplicationOperationContractRecord::new(
            "Schema".to_owned(),
            "Change".to_owned(),
            WorthQueryPortableTypeIdentity::from_untrusted("Input".to_owned()),
            vec![],
            vec![],
            vec![],
            None,
            None,
        )
        .with_body(Some(
            ApplicationOperationBodyProgram::from_untrusted_in_scope(
                "Item".to_owned(),
                vec![],
                Some(ApplicationOperationBodyCreation::from_untrusted(
                    "Audit".to_owned(),
                    "audit".to_owned(),
                    vec![step],
                )),
                vec![],
            ),
        ))
        .reconstruction_work()
    };
    let literal = record(Field::Literal {
        field: target.clone(),
        binding: binding.clone(),
        value: value.clone(),
    });
    let principal = record(Field::PrincipalIdentity {
        field: target,
        binding,
        scalar_family: ScalarAspectType::UInt64,
        mutation_binding: "ChangeBinding".to_owned(),
        principal_binding: "PrincipalBinding".to_owned(),
        source: ApplicationFieldBindingLocus::new("Principal", "Identity", "Id"),
    });
    let source_bytes: u64 = [
        "ChangeBinding",
        "PrincipalBinding",
        "Principal",
        "Identity",
        "Id",
    ]
    .into_iter()
    .map(crate::package::reconstruction_text_bytes)
    .sum();
    assert_eq!(
        principal.logical_bytes + value.semantic_byte_width() as u64,
        literal.logical_bytes + source_bytes + 2
    );
    assert_eq!(principal.nested_entries, literal.nested_entries + 5);
}
