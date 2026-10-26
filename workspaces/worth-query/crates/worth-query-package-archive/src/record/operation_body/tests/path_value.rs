//! v33 preserves target meaning and charges the same nested path entries on decode.
use super::*;
use worth_query_declaration::facade::{
    application_query::*,
    application_schema::{
        ApplicationFieldPresence, ApplicationOperationBodyPathFieldSource,
        ApplicationOperationBodyPathValueAssignment,
    },
};
#[test]
fn path_value_v33_roundtrips_and_old_version_cannot_consume_the_assignment() {
    for direction in [
        None,
        Some(RelationDirection::Reverse),
        Some(RelationDirection::Forward),
    ] {
        let terminal = if direction.is_none() { "Row" } else { "Parent" };
        let path = ApplicationQueryRootPathMeaning::from_untrusted_parts(
            WorthQueryPortableApplicationQueryRootPathParts {
                start_entity: "Row".into(),
                terminal_entity: terminal.into(),
                guards: vec![],
                steps: direction
                    .map(|direction| match direction {
                        RelationDirection::Reverse => {
                            ApplicationQueryRootPathStep::from_untrusted_fields(
                                "ParentRows".into(),
                                "Parent".into(),
                                "Row".into(),
                                direction,
                            )
                        }
                        RelationDirection::Forward => {
                            ApplicationQueryRootPathStep::from_untrusted_fields(
                                "RowParent".into(),
                                "Row".into(),
                                "Parent".into(),
                                direction,
                            )
                        }
                    })
                    .into_iter()
                    .collect(),
            },
        );
        let body = ApplicationOperationBodyProgram::from_untrusted_in_scope(
            "Row".into(),
            vec![],
            None,
            vec![],
        )
        .with_untrusted_path_value_assignments(vec![
            ApplicationOperationBodyPathValueAssignment {
                target: ApplicationOperationBodyPathFieldSource {
                    path,
                    field: ApplicationFieldBindingLocus::new(terminal, "Facts", "Revision"),
                    presence: ApplicationFieldPresence::Required,
                },
                binding: WorthQueryPortableTypeIdentity::from_untrusted("worth.rust.u64".into()),
                value: "next".into(),
            },
        ]);
        let mut output = BinaryOutput::with_capacity(2048);
        write(&mut output, &body).unwrap();
        let mut bytes = output.into_bytes();
        assert_eq!(&bytes[..2], &33_u16.to_be_bytes());
        let budget = |bytes: &[u8]| {
            RecordDecodeAttempt::begin(
                Default::default(),
                bytes.len() as u64,
                WorthQueryPackageArchiveLimits::DEFAULT,
            )
            .unwrap()
        };
        let mut input = BinaryInput::new(&bytes);
        let mut decoded = budget(&bytes);
        assert_eq!(decode(&mut input, &mut decoded).unwrap(), body);
        assert!(input.is_finished());
        let mut measure = crate::binary_encoding::BinaryEncodingMeasure::default();
        write(&mut measure, &body).unwrap();
        assert_eq!(measure.nested_entries(), decoded.finish().nested_entries());
        bytes[..2].copy_from_slice(&32_u16.to_be_bytes());
        let mut input = BinaryInput::new(&bytes);
        assert_ne!(decode(&mut input, &mut budget(&bytes)).unwrap(), body);
        assert!(!input.is_finished());
        bytes[..2].copy_from_slice(&33_u16.to_be_bytes());
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
