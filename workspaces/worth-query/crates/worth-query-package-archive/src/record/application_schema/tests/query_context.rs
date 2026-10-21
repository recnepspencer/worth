//! Codec carriage of a full query source is descriptive, never readmission.
use super::*;
use crate::{binary_input::BinaryInput, record::operation_body};
use worth_query_declaration::facade::{
    application_query::{ApplicationQueryCardinality, ErasedApplicationQueryDefinition},
    application_schema::{
        ApplicationOperationBodyProgram as Body,
        ApplicationOperationBodyRequirementExpression as Expression,
        ApplicationOperationBodyRequirementOperand as Operand,
        ApplicationOperationBodyRequirementProgram as Requirement,
        ApplicationOperationBodyRequirementSource as Source,
    },
};

fn context_body() -> Body {
    let record = fixture::complete_untrusted_schema_record();
    let mut query = record
        .members()
        .iter()
        .find_map(|member| {
            if let ApplicationSchemaMember::ApplicationQuery { definition } = member {
                Some(definition.parts().clone())
            } else {
                None
            }
        })
        .unwrap();
    query.cardinality = ApplicationQueryCardinality::ExactlyOne;
    query.parameters.clear();
    query.predicates.clear();
    let expression =
        worth_foundational::expression_api::expressions().reading_within(Expression::PROFILE);
    let requirement = Requirement::from_untrusted(
        "source-value".into(),
        vec![Operand::from_untrusted(
            "context".into(),
            Source::QueryContext {
                mutation_binding: "archive.tests.check.v1".into(),
                query: Box::new(ErasedApplicationQueryDefinition::from_untrusted_parts(
                    query,
                )),
            },
        )],
        None,
        expression
            .parse("context.value > exact_cast<UInt64>(0)")
            .unwrap()
            .encode(),
        "context".into(),
    )
    .unwrap();
    Body::from_untrusted_in_scope("Entity".into(), vec![], None, vec![requirement])
}

#[test]
fn v25_retains_full_query_context_and_older_profiles_refuse_it() {
    let body = context_body();
    let mut output = BinaryOutput::with_capacity(4096);
    operation_body::write(&mut output, &body).unwrap();
    let bytes = output.into_bytes();
    assert_eq!(&bytes[..2], &25_u16.to_be_bytes());
    let mut budget = RecordDecodeAttempt::begin(
        Default::default(),
        bytes.len() as u64,
        WorthQueryPackageArchiveLimits::DEFAULT,
    )
    .unwrap();
    let mut input = BinaryInput::new(&bytes);
    assert_eq!(
        operation_body::decode(&mut input, &mut budget).unwrap(),
        body
    );
    assert!(input.is_finished());
    let mut measure = crate::binary_encoding::BinaryEncodingMeasure::default();
    operation_body::write(&mut measure, &body).unwrap();
    assert_eq!(measure.nested_entries(), budget.finish().nested_entries());
    for version in [7_u16, 24] {
        let mut old = bytes.clone();
        old[..2].copy_from_slice(&version.to_be_bytes());
        let mut budget = RecordDecodeAttempt::begin(
            Default::default(),
            old.len() as u64,
            WorthQueryPackageArchiveLimits::DEFAULT,
        )
        .unwrap();
        assert_eq!(
            operation_body::decode(&mut BinaryInput::new(&old), &mut budget)
                .unwrap_err()
                .kind(),
            WorthQueryPackageArchiveDenialKind::UnsupportedRecordVariant
        );
    }
}

#[test]
fn operation_contract_query_depth_is_symmetric_on_encode_and_decode() {
    use crate::record::application_operation_contract as contract;
    use worth_query_declaration::facade::portable_identity::WorthQueryPortableTypeIdentity;
    use worth_query_installation::facade::{
        WorthQueryPortableApplicationOperationContractParts,
        WorthQueryPortableApplicationOperationContractRecord,
    };
    for body in [context_body(), payload_context_body()] {
        let record = WorthQueryPortableApplicationOperationContractRecord::from_untrusted_parts(
            WorthQueryPortableApplicationOperationContractParts {
                schema: "ArchiveSchema".into(),
                operation: "Check".into(),
                input_type: WorthQueryPortableTypeIdentity::from_untrusted("archive.input".into()),
                graph_reads: vec![],
                touches: vec![],
                emissions: vec![],
                external_effect: None,
                reconciliation: None,
                body: Some(body),
            },
        );
        let exact = WorthQueryPackageArchiveLimits::DEFAULT.with_maximum_nesting_depth(3);
        let narrow = exact.with_maximum_nesting_depth(2);
        assert_eq!(
            contract::payload_encoding_work(&record, narrow)
                .unwrap_err()
                .kind(),
            WorthQueryPackageArchiveDenialKind::NestingDepthBudgetExceeded
        );
        let work = contract::payload_encoding_work(&record, exact).unwrap();
        let mut output = BinaryOutput::with_capacity(work.payload_bytes() as usize);
        contract::write_payload(&record, &mut output).unwrap();
        let bytes = output.into_bytes();
        for (limits, accepted) in [(narrow, false), (exact, true)] {
            let mut budget =
                RecordDecodeAttempt::begin(Default::default(), bytes.len() as u64, limits).unwrap();
            let decoded = contract::decode_payload(&mut BinaryInput::new(&bytes), &mut budget);
            if accepted {
                assert_eq!(
                    decoded.unwrap(),
                    WorthQueryPortablePackageRecord::ApplicationOperationContract(record.clone())
                );
            } else {
                assert_eq!(
                    decoded.unwrap_err().kind(),
                    WorthQueryPackageArchiveDenialKind::NestingDepthBudgetExceeded
                );
            }
        }
    }
}

fn payload_context_body() -> Body {
    use worth_query_declaration::facade::{
        application_schema::{
            ApplicationExternalEffectProtocol,
            ApplicationOperationBodyExternalEmission as Emission,
            ApplicationOperationBodyExternalPayloadSource as PayloadSource,
        },
        portable_identity::WorthQueryPortableTypeIdentity as Identity,
    };
    let body = context_body();
    let Source::QueryContext {
        mutation_binding,
        query,
    } = body.requirements()[0].operands()[0].source()
    else {
        unreachable!()
    };
    Body::from_untrusted_in_scope("Entity".into(), vec![], None, vec![])
        .with_untrusted_external_emission(Some(Emission {
            source: PayloadSource::QueryContext {
                mutation_binding: mutation_binding.clone(),
                query: query.clone(),
            },
            effect: "Notice".into(),
            input_binding: Identity::from_untrusted("input".into()),
            payload_binding: Identity::from_untrusted("payload".into()),
            protocol: ApplicationExternalEffectProtocol::new(
                worth_foundational::facade::BoundaryProtocolIdentity::new("context.notice"),
                worth_foundational::facade::BoundaryProtocolVersion::new(1),
            ),
            maximum_payload_bytes: 512,
            condition: None,
        }))
}

#[test]
fn v27_carries_an_unguarded_payload_query_and_v26_cannot_reinterpret_it() {
    let body = payload_context_body();
    assert!(body.requirements().is_empty());
    assert!(body.requires_query_context());
    let mut output = BinaryOutput::with_capacity(4096);
    operation_body::write(&mut output, &body).unwrap();
    let mut bytes = output.into_bytes();
    assert_eq!(&bytes[..2], &27_u16.to_be_bytes());
    let mut budget = RecordDecodeAttempt::begin(
        Default::default(),
        bytes.len() as u64,
        WorthQueryPackageArchiveLimits::DEFAULT,
    )
    .unwrap();
    let mut input = BinaryInput::new(&bytes);
    assert_eq!(
        operation_body::decode(&mut input, &mut budget).unwrap(),
        body
    );
    assert!(input.is_finished());
    let mut measure = crate::binary_encoding::BinaryEncodingMeasure::default();
    operation_body::write(&mut measure, &body).unwrap();
    assert_eq!(measure.nested_entries(), budget.finish().nested_entries());
    bytes[..2].copy_from_slice(&26_u16.to_be_bytes());
    let mut budget = RecordDecodeAttempt::begin(
        Default::default(),
        bytes.len() as u64,
        WorthQueryPackageArchiveLimits::DEFAULT,
    )
    .unwrap();
    let mut input = BinaryInput::new(&bytes);
    let old = operation_body::decode(&mut input, &mut budget).unwrap();
    assert!(!old.requires_query_context());
    assert!(
        !input.is_finished(),
        "outer exact-consumption admission rejects the downgrade"
    );
}
