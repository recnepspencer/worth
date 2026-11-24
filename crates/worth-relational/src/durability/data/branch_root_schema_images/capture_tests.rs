//! Full image meaning and admission survive shared DATA aliases.
use super::*;
use crate::config::data::{CascadeDeletePolicy, CrossContextPolicy};
use sha2::{Digest, Sha256};
use std::sync::Arc;
use worth_execution::{CancellationSource, ExecutionRequest, SerialMemoryBudget, SerialRequest};

fn original_schema() -> RelationalSchemaRegistry {
    use crate::schema::data::*;
    RelationalSchemaRegistry::new()
        .register_entity_kind(EntityKindRegistration {
            kind_id: KindId(1),
            kind_name: "unused_entity".into(),
            schema_id: SchemaId("image.schema".into()),
            schema_version_id: SchemaVersionId(1),
            aspect_contract_declarations: KindAspectContractDeclarations::new(Vec::new()),
        })
        .unwrap()
        .register_relation_kind(RelationKindRegistration {
            kind_id: KindId(2),
            kind_name: "unused_relation".into(),
            schema_id: SchemaId("image.schema".into()),
            schema_version_id: SchemaVersionId(1),
            cross_context_policy: CrossContextPolicy::AllowExplicit,
            cascade_delete_policy: CascadeDeletePolicy::RetainDanglingForAudit,
            aspect_contract_declarations: KindAspectContractDeclarations::new(Vec::new()),
            relation_integrity: RelationIntegrityDeclarations::new(
                vec![EndpointKindContractDeclaration {
                    contract_id: "unused_endpoint".into(),
                    allowed_source_kinds: vec![KindId(1)],
                    allowed_target_kinds: vec![KindId(1)],
                    self_edges_allowed: false,
                    cross_context_policy: CrossContextPolicy::AllowExplicit,
                }],
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
            ),
        })
        .unwrap()
}

fn source(registry: &RelationalSchemaRegistry) -> SchemaImageSource<'_> {
    use crate::schema::data::{
        runtime_descriptor_semantics_policy, schema_authority_snapshot_digest_bytes,
    };
    SchemaImageSource {
        registry,
        retained_aspect_contracts: &[],
        schema_root: schema_authority_snapshot_digest_bytes(&registry.authority_snapshot()),
        descriptor_semantics_version: runtime_descriptor_semantics_policy().current_write_version(),
    }
}

#[test]
fn full_schema_capture_keeps_unused_constraints_wire_and_last_alias_admission() {
    let original = original_schema();
    let ordinary = RecordedBranchSchemaImage::capture(source(&original)).unwrap();
    let budget = SerialMemoryBudget::new(Some(1024 * 1024));
    let host = SerialRequest::from_memory(budget.clone(), CancellationSource::new().token(), None);
    let request = ExecutionRequest::serial(&host);
    let image =
        RecordedBranchSchemaImage::capture_for_request(source(&original), &request).unwrap();
    let charged = budget.charged_memory_bytes();
    assert!(charged > 0);
    assert_eq!(image, ordinary);
    assert_eq!(
        rmp_serde::to_vec(&image).unwrap(),
        rmp_serde::to_vec(&ordinary).unwrap()
    );
    let encoded = rmp_serde::to_vec(&image).unwrap();
    let decoded: RecordedBranchSchemaImage = rmp_serde::from_slice(&encoded).unwrap();
    assert_eq!(image, decoded);
    let (registry, _) = decoded.readmit_registry().unwrap();
    assert_eq!(&registry, &original);
    assert_eq!(
        registry.relation_kinds[&KindId(2)]
            .relation_integrity
            .endpoint_kind_contracts
            .len(),
        1
    );
    let data = &image.0;
    let legacy = rmp_serde::to_vec(&(
        data.format_version,
        data.schema_root,
        data.descriptor_semantics_version,
        &data.entity_registrations,
        &data.relation_registrations,
        &data.retained_aspect_contracts,
    ))
    .unwrap();
    let mut digest = Sha256::new();
    digest.update(b"worth.relational.branch-root-schema-image.v1\0");
    digest.update(legacy);
    digest.update(b"worth.native.content-schema-carrier.v1\0");
    digest.update(
        rmp_serde::to_vec(&crate::content::RelationalContentSchemaSnapshot::default()).unwrap(),
    );
    assert_eq!(image.carrier_digest(), <[u8; 32]>::from(digest.finalize()));
    let alias = image.clone();
    assert!(Arc::ptr_eq(&image.0, &alias.0));
    drop(image);
    assert_eq!(budget.charged_memory_bytes(), charged);
    drop(alias);
    assert_eq!(budget.charged_memory_bytes(), 0);
}

#[test]
fn schema_capture_refuses_original_request_memory_and_cancellation_before_copy() {
    let original = original_schema();
    let calibration = SerialMemoryBudget::new(Some(64 * 1024));
    let host =
        SerialRequest::from_memory(calibration.clone(), CancellationSource::new().token(), None);
    let request = ExecutionRequest::serial(&host);
    let mut controller = None;
    request
        .run_native_with_checkpoint(|_| controller = Some(calibration.charged_memory_bytes()))
        .unwrap();
    assert_eq!(calibration.charged_memory_bytes(), 0);
    let minimum = std::alloc::Layout::new::<(usize, usize)>()
        .extend(std::alloc::Layout::new::<SchemaImageData>())
        .unwrap()
        .0
        .pad_to_align()
        .size() as u64;
    let budget =
        SerialMemoryBudget::new(Some(controller.unwrap().checked_add(minimum).unwrap() - 1));
    let host = SerialRequest::from_memory(budget.clone(), CancellationSource::new().token(), None);
    let request = ExecutionRequest::serial(&host);
    assert!(matches!(
        RecordedBranchSchemaImage::capture_for_request(source(&original), &request),
        Err(crate::durability::data::recovery_stop::RelationalRecoveryStop::Request(
            crate::execution::denial_cause::RelationalExecutionDenialCause::PolicyMemoryExhausted {
                requested, admitted, ancestor: 0,
            }
        )) if requested >= minimum && admitted < minimum
    ));
    assert_eq!(budget.charged_memory_bytes(), 0);
    let cancellation = CancellationSource::new();
    let host = SerialRequest::from_memory(budget.clone(), cancellation.token(), None);
    cancellation.cancel();
    let request = ExecutionRequest::serial(&host);
    assert!(matches!(
        RecordedBranchSchemaImage::capture_for_request(source(&original), &request),
        Err(
            crate::durability::data::recovery_stop::RelationalRecoveryStop::Request(
                crate::execution::denial_cause::RelationalExecutionDenialCause::Cancelled
            )
        )
    ));
    assert_eq!(budget.charged_memory_bytes(), 0);
}
