//! New guard meaning carries native identity/reverse paths with exact decode accounting.
use super::*;
use worth_query_declaration::facade::{
    application_query::{
        ApplicationQueryRootPathMeaning, ApplicationQueryRootPathStep as Step,
        RelationDirection as Direction, WorthQueryPortableApplicationQueryRootPathParts as Parts,
    },
    application_schema::{
        ApplicationFieldPresence as Presence, ApplicationOperationBodyPathFieldSource as Source,
        ApplicationOperationBodyPathInputExpectation as Guard,
    },
};
#[test]
fn path_input_v32_roundtrips_identity_and_reverse_paths_and_cannot_fully_downgrade() {
    for identity in [true, false] {
        let source = Source {
            path: ApplicationQueryRootPathMeaning::from_untrusted_parts(Parts {
                start_entity: "Row".into(),
                terminal_entity: if identity { "Row" } else { "Parent" }.into(),
                steps: if identity {
                    vec![]
                } else {
                    vec![Step::from_untrusted_fields(
                        "ParentRows".into(),
                        "Parent".into(),
                        "Row".into(),
                        Direction::Reverse,
                     worth_query_declaration::facade::application_query::ApplicationQueryRootPathTraversal::OneHop,)]
                },
                guards: vec![],
            }),
            field: if identity {
                ApplicationFieldBindingLocus::new("Row", "Facts", "Identity")
            } else {
                ApplicationFieldBindingLocus::new("Parent", "Facts", "Company")
            },
            presence: if identity {
                Presence::Required
            } else {
                Presence::Optional
            },
        };
        let body = ApplicationOperationBodyProgram::from_untrusted_in_scope(
            "Row".into(),
            vec![ApplicationOperationBodyScopeField::Assign {
                field: ApplicationFieldBindingLocus::new("Row", "Facts", "Value"),
                binding: WorthQueryPortableTypeIdentity::from_untrusted("worth.rust.u64".into()),
                value: AspectValue::UInt64(42),
            }],
            None,
            vec![],
        )
        .with_untrusted_path_input_expectations(vec![Guard {
            source,
            binding: WorthQueryPortableTypeIdentity::from_untrusted("worth.rust.string".into()),
            scalar_family: ScalarAspectType::String,
            member: "company".into(),
        }]);
        let mut output = BinaryOutput::with_capacity(2048);
        write(&mut output, &body).unwrap();
        let mut bytes = output.into_bytes();
        assert_eq!(&bytes[..2], &32_u16.to_be_bytes());
        let budget = |bytes: &[u8]| {
            RecordDecodeAttempt::begin(
                Default::default(),
                bytes.len() as u64,
                WorthQueryPackageArchiveLimits::DEFAULT,
            )
            .unwrap()
        };
        let mut input = BinaryInput::new(&bytes);
        let mut decoded_work = budget(&bytes);
        assert_eq!(decode(&mut input, &mut decoded_work).unwrap(), body);
        assert!(input.is_finished());
        let mut measure = crate::binary_encoding::BinaryEncodingMeasure::default();
        write(&mut measure, &body).unwrap();
        assert_eq!(
            measure.nested_entries(),
            decoded_work.finish().nested_entries()
        );
        bytes[..2].copy_from_slice(&31_u16.to_be_bytes());
        let mut input = BinaryInput::new(&bytes);
        assert_ne!(decode(&mut input, &mut budget(&bytes)).unwrap(), body);
        assert!(!input.is_finished());
        bytes[..2].copy_from_slice(&32_u16.to_be_bytes());
        let presence = bytes.len() - 2;
        bytes[presence..].copy_from_slice(&99_u16.to_be_bytes());
        assert_eq!(
            decode(&mut BinaryInput::new(&bytes), &mut budget(&bytes))
                .unwrap_err()
                .kind(),
            Kind::UnsupportedRecordVariant
        );
    }
}
