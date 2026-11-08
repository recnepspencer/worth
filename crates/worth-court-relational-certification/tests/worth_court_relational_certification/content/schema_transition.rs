//! Changing executable content authority rechecks bytes before any publication.
use super::{fixture::Codec, original, recovery::publish, transaction::*};
use std::{any::Any, sync::Arc};
use worth_execution::{CancellationSource, ExecutionRequest};
use worth_foundational::facade::*;
use worth_relational::facade::{content::*, identity::KindId, mvcc::*, runtime::*, schema::*};

#[derive(Debug)]
struct RestrictedCodec {
    codec: Codec,
    maximum: usize,
    reject: bool,
}
impl RelationalContentCodec for RestrictedCodec {
    fn format(&self) -> &ContentFormat {
        &self.codec.descriptor
    }
    fn maximum_frame_bytes(&self) -> usize {
        self.maximum
    }
    fn payload_length(
        &self,
        input: &dyn Any,
        control: &mut RelationalContentControl<'_, '_, '_>,
    ) -> Result<usize, RelationalContentDenial> {
        self.codec.payload_length(input, control)
    }
    fn encode(
        &self,
        input: &dyn Any,
        sink: &mut RelationalContentSink<'_>,
        control: &mut RelationalContentControl<'_, '_, '_>,
    ) -> Result<(), RelationalContentDenial> {
        self.codec.encode(input, sink, control)
    }
    fn validate(
        &self,
        payload: &[u8],
        control: &mut RelationalContentControl<'_, '_, '_>,
    ) -> Result<(), RelationalContentDenial> {
        if self.reject {
            Err(RelationalContentDenial::NonCanonicalContent)
        } else {
            self.codec.validate(payload, control)
        }
    }
    fn verify_encoding(
        &self,
        input: &dyn Any,
        payload: &[u8],
        control: &mut RelationalContentControl<'_, '_, '_>,
    ) -> Result<(), RelationalContentDenial> {
        self.codec.verify_encoding(input, payload, control)
    }
}

fn restricted(maximum: usize, reject: bool) -> Arc<RestrictedCodec> {
    Arc::new(RestrictedCodec {
        codec: Codec {
            descriptor: ContentFormat::new("sample-grid", 1).unwrap(),
            alter: false,
        },
        maximum,
        reject,
    })
}

fn transition() -> ProposedSchemaTransition {
    let schema = SchemaId("test".into());
    ProposedSchemaTransition {
        source_schema_id: schema.clone(),
        source_schema_version_id: SchemaVersionId(1),
        target_schema_id: schema.clone(),
        target_schema_version_id: SchemaVersionId(2),
        diff_atoms: vec![SchemaDiffAtom::new(
            SchemaElementRef::new(
                SchemaElementKind::Field,
                schema,
                SchemaVersionId(2),
                Some(KindId(1)),
                "annotation",
            ),
            vec![
                SchemaStratum::StructuralShape,
                SchemaStratum::PublicationContract,
            ],
            SchemaPublicationImpact::ObservableSurfaceChanged,
            SchemaSubscriberImpact::ConsumableSurfaceChanged,
            HistoricalInterpretationSensitivity::NotSensitive,
            SchemaDiffDetail::AddedField {
                field: FieldKey::new("annotation").unwrap(),
                required: false,
                default_expression: Some("null".into()),
            },
        )
        .with_boundary_visibility_proof(
            SubscriberBoundaryVisibility::VisibleSemanticallyIgnorable,
        )],
    }
}

#[test]
fn content_reduced_ceiling_refuses_transition_admission_and_record_readmission() {
    let cancel = CancellationSource::new();
    let request = original(&cancel);
    let execution = ExecutionRequest::serial(&request);
    let (runtime, contract) = fixture();
    let (id, _) = publish(&runtime, &execution, &contract);
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let frame = basis.read_content(&execution, id, FORMAT).unwrap();
    let codec = restricted(74, false);
    let (target, _) = fixture_registry(codec.clone(), SchemaVersionId(2));
    let refusal = runtime
        .begin_branch_schema_transition(
            &basis,
            transition(),
            Some(SchemaReconciliationPolicy::PreserveInformation),
            target,
        )
        .unwrap_err();
    assert_eq!(
        refusal.kind(),
        RelationalSchemaTransitionAdmissionDenialKind::TargetContractMismatch
    );
    let mut formats = RelationalContentFormatRegistry::default();
    formats.register(FORMAT, codec).unwrap();
    assert_eq!(
        RelationalContentFrame::readmit_record(
            &execution,
            formats.get(FORMAT).unwrap(),
            &frame.record()
        )
        .unwrap_err(),
        RelationalContentDenial::SizeLimit {
            observed: 75,
            maximum: 74
        }
    );
    assert_eq!(
        runtime
            .observe_branch(&runtime.main_branch_identity())
            .unwrap()
            .1
            .descriptor(),
        basis.descriptor()
    );
}

#[test]
fn content_new_installed_validator_cannot_publish_previously_admitted_bytes() {
    let cancel = CancellationSource::new();
    let request = original(&cancel);
    let execution = ExecutionRequest::serial(&request);
    let (runtime, contract) = fixture();
    let (id, _) = publish(&runtime, &execution, &contract);
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let (target, _) = fixture_registry(
        restricted(MAXIMUM_CONTENT_FRAME_BYTES, true),
        SchemaVersionId(2),
    );
    let mut transaction = runtime
        .begin_branch_schema_transition(
            &basis,
            transition(),
            Some(SchemaReconciliationPolicy::PreserveInformation),
            target,
        )
        .unwrap();
    transaction
        .push_batch(batch(&contract, id), execution.allocation_policy())
        .unwrap();
    let refusal = transaction.commit(&execution, &runtime).unwrap_err();
    assert!(
        matches!(
            refusal,
            TransactionCommitError::Conflict {
                error: CommitConflict {
                    class: ConflictClass::ContentAdmission {
                        denial: RelationalContentDenial::NonCanonicalContent
                    },
                    ..
                },
                ..
            }
        ),
        "{refusal:?}"
    );
    assert_eq!(
        runtime
            .observe_branch(&runtime.main_branch_identity())
            .unwrap()
            .1
            .descriptor(),
        basis.descriptor()
    );
}

#[test]
fn content_declared_ceiling_refuses_instead_of_silently_narrowing_portable_meaning() {
    let mut formats = RelationalContentFormatRegistry::default();
    assert_eq!(
        formats.register(FORMAT, restricted(16 * 1024 * 1024 + 1, false)),
        Err(RelationalContentDenial::SizeLimit {
            observed: 16 * 1024 * 1024 + 1,
            maximum: 16 * 1024 * 1024
        })
    );
    assert!(formats.snapshot().is_empty());
    assert_eq!(
        formats.register(FORMAT, restricted(0, false)),
        Err(RelationalContentDenial::InvalidFormat)
    );
    assert!(formats.snapshot().is_empty());
    formats
        .register(FORMAT, restricted(16 * 1024 * 1024, false))
        .unwrap();
    assert_eq!(
        formats.get(FORMAT).unwrap().maximum_frame_bytes(),
        16 * 1024 * 1024
    );
}

#[test]
fn content_typed_handoff_requires_exact_frozen_contract_under_reused_format_identity() {
    let cancel = CancellationSource::new();
    let request = original(&cancel);
    let execution = ExecutionRequest::serial(&request);
    let mut old = RelationalContentFormatRegistry::default();
    old.register(FORMAT, restricted(1024, false)).unwrap();
    let frame = RelationalContentFrame::encode(
        &execution,
        old.get(FORMAT).unwrap(),
        &super::fixture::grid(),
    )
    .unwrap();
    assert_eq!(frame.bytes().len(), 75);
    assert_eq!(
        frame.require_contract(&execution, "sample-grid", 1, 1024),
        Ok(())
    );
    for (name, version, maximum, expected) in [
        (
            "different-grid",
            1,
            1024,
            RelationalContentDenial::FormatMismatch,
        ),
        (
            "sample-grid",
            2,
            1024,
            RelationalContentDenial::UnsupportedVersion,
        ),
        (
            "sample-grid",
            1,
            2048,
            RelationalContentDenial::FormatMismatch,
        ),
        (
            "sample-grid",
            1,
            512,
            RelationalContentDenial::FormatMismatch,
        ),
    ] {
        let mut new = RelationalContentFormatRegistry::default();
        new.register(
            FORMAT,
            Arc::new(RestrictedCodec {
                codec: Codec {
                    descriptor: ContentFormat::new(name, version).unwrap(),
                    alter: false,
                },
                maximum,
                reject: false,
            }),
        )
        .unwrap();
        let expected_contract = new.get(FORMAT).unwrap();
        assert!(frame.bytes().len() < expected_contract.maximum_frame_bytes());
        assert_eq!(
            frame.require_contract(
                &execution,
                expected_contract.descriptor().name(),
                expected_contract.descriptor().version(),
                expected_contract.maximum_frame_bytes()
            ),
            Err(expected)
        );
    }
    assert_eq!(
        frame.require_contract(&execution, "sample-grid", 1, 74),
        Err(RelationalContentDenial::SizeLimit {
            observed: 75,
            maximum: 74
        })
    );
    cancel.cancel();
    assert!(matches!(
        frame.require_contract(&execution, "different-grid", 2, 2048),
        Err(RelationalContentDenial::Execution(_))
    ));
}

#[test]
fn content_schema_extension_preserves_actual_format_handles_and_refuses_replacement() {
    let first = RelationalSchemaRegistry::new()
        .register_content_format(FORMAT, restricted(1024, false))
        .unwrap()
        .bind_content_field(CanonicalDigestId::new([0xE1; 32]), FORMAT)
        .unwrap();
    let other_id = CanonicalDigestId::new([0xE2; 32]);
    let second = RelationalSchemaRegistry::new()
        .register_content_format(
            other_id,
            Arc::new(RestrictedCodec {
                codec: Codec {
                    descriptor: ContentFormat::new("second-grid", 1).unwrap(),
                    alter: false,
                },
                maximum: 2048,
                reject: false,
            }),
        )
        .unwrap()
        .bind_content_field(CanonicalDigestId::new([0xE3; 32]), other_id)
        .unwrap();
    let expected_first = first.authority_snapshot().content_formats.clone();
    let expected_second = second.authority_snapshot().content_formats.clone();
    let combined = first.clone().extend(second).unwrap();
    let actual = combined.authority_snapshot().content_formats.clone();
    assert_eq!(actual.formats().len(), 2);
    assert_eq!(actual.fields().len(), 2);
    for expected in expected_first
        .formats()
        .iter()
        .chain(expected_second.formats())
    {
        assert!(actual.formats().contains(expected));
    }
    for expected in expected_first
        .fields()
        .iter()
        .chain(expected_second.fields())
    {
        assert!(actual.fields().contains(expected));
    }
    assert!(combined.extend(first).is_err());
}
