//! Owner comparisons over lowered effects; public courts cover actual commit custody.
use super::candidate::AuthoredCorrectionCandidateDenial;
use super::*;
use worth_query_admission::facade::authenticated_principal::{
    WorthQueryCancellationSource, WorthQueryRequestInterruption, WorthQueryRequestScope,
};
use worth_query_declaration::facade::{
    application_aftermath::DeclaredAuthoredCorrectionParts,
    application_schema::ApplicationFieldPresence,
    portable_identity::WorthQueryPortableTypeIdentity,
};
use worth_relational::facade::{
    identity::{KindId, PartitionId},
    symbols::ClientKey,
    transactions::{
        CreateIntent, CreatedEntityRef, DeleteEntityIntent, EntityMutationIntent, EntityReference,
        EntitySpec, MutationIntent, WorkerIntentBatch,
    },
};

fn prepared(posture: Posture) -> PreparedAuthoredIdentity {
    PreparedAuthoredIdentity {
        declaration: DeclaredAuthoredCorrection::from_untrusted_parts(
            DeclaredAuthoredCorrectionParts {
                family: WorthQueryPortableTypeIdentity::from_untrusted(
                    "authored-number.v1".to_owned(),
                ),
                logical_key: Box::new(ApplicationSchemaMember::Field {
                    entity: "Number".to_owned(),
                    aspect: "Facts".to_owned(),
                    field: "Key".to_owned(),
                    presence: ApplicationFieldPresence::Required,
                    scalar_family: worth_foundational::facade::ScalarAspectType::String,
                    value_type: "worth.string".to_owned(),
                    canonical: None,
                    unit: None,
                    frame: None,
                    writable: false,
                    equality_queryable: false,
                    unique: false,
                }),
                binding_unit: None,
                source_role: "original".to_owned(),
                inverse_role: "inverse".to_owned(),
                source_posture: posture,
            },
        ),
        original: EntityId::new(PartitionId::main(), 1, 1),
        locator: AspectFieldLocator::new(
            LocatorAuthority::Planned,
            AspectKey::new("Facts").unwrap(),
            CanonicalFieldPath::single(FieldKey::new("Key").unwrap()),
        ),
        key: AspectValue::String("logical-number".into()),
    }
}
fn created(client_key: &str) -> CreatedEntityRef {
    CreatedEntityRef {
        partition_id: PartitionId::main(),
        kind_id: KindId(1),
        client_key: ClientKey::raw(client_key),
    }
}
fn creation(
    identity: &PreparedAuthoredIdentity,
    reference: &CreatedEntityRef,
    key: AspectValue,
) -> WorkerIntentBatch {
    let fields = worth_relational::facade::transactions::AspectFieldPatch::from_locator(
        identity.locator.clone(),
        key,
    );
    WorkerIntentBatch::new("candidate").push(MutationIntent::Create(CreateIntent::Entity(
        EntitySpec {
            partition_id: reference.partition_id,
            kind_id: reference.kind_id,
            client_key: reference.client_key.clone(),
            fields,
        },
    )))
}
#[test]
fn same_key_cannot_substitute_another_physical_identity_or_generation() {
    let prepared = prepared(Posture::Create);
    let cancellation = WorthQueryCancellationSource::new();
    let request = WorthQueryRequestScope::new(None, cancellation.token());
    let deletion = |entity_id| {
        WorkerIntentBatch::new("retire").push(MutationIntent::Entity(EntityMutationIntent::Delete(
            DeleteEntityIntent { entity_id },
        )))
    };
    let batch = deletion(prepared.original);
    let exact = EntityReference::Existing(prepared.original);
    assert!(prepared
        .validate_candidate(
            [("inverse", Posture::Retire, "Number", &exact)].into_iter(),
            &batch,
            &request
        )
        .is_ok());
    for different in [
        EntityId::new(PartitionId::main(), 2, 1),
        EntityId::new(PartitionId::main(), 1, 2),
    ] {
        // The retained family and logical key remain identical; only the native identity changes.
        let wrong = EntityReference::Existing(different);
        assert!(matches!(
            prepared.validate_candidate(
                [("inverse", Posture::Retire, "Number", &wrong)].into_iter(),
                &deletion(different),
                &request
            ),
            Err(AuthoredCorrectionCandidateDenial::Identity(_))
        ));
        assert!(matches!(
            prepared.validate_candidate(
                [("inverse", Posture::Retire, "Number", &exact)].into_iter(),
                &deletion(different),
                &request
            ),
            Err(AuthoredCorrectionCandidateDenial::Identity(_))
        ));
    }
    assert!(matches!(
        prepared.validate_candidate(
            [("inverse", Posture::Retire, "Number", &exact)].into_iter(),
            &WorkerIntentBatch::new("missing-delete"),
            &request
        ),
        Err(AuthoredCorrectionCandidateDenial::Identity(_))
    ));
}
#[test]
fn restoration_matches_its_actual_lowered_create_and_original_key() {
    let prepared = prepared(Posture::Retire);
    let cancellation = WorthQueryCancellationSource::new();
    let request = WorthQueryRequestScope::new(None, cancellation.token());
    let reference = created("new-allocation");
    let target = EntityReference::Created(reference.clone());
    let role = || [("inverse", Posture::Create, "Number", &target)].into_iter();
    let lawful = creation(&prepared, &reference, prepared.key.clone());
    assert!(prepared
        .validate_candidate(role(), &lawful, &request)
        .is_ok());
    let wrong_key = creation(
        &prepared,
        &reference,
        AspectValue::String("changed-key".into()),
    );
    assert!(matches!(
        prepared.validate_candidate(role(), &wrong_key, &request),
        Err(AuthoredCorrectionCandidateDenial::Identity(_))
    ));
    let wrong_allocation = creation(
        &prepared,
        &created("different-allocation"),
        prepared.key.clone(),
    );
    assert!(matches!(
        prepared.validate_candidate(role(), &wrong_allocation, &request),
        Err(AuthoredCorrectionCandidateDenial::Identity(_))
    ));
    let old = EntityReference::Existing(prepared.original);
    assert!(matches!(
        prepared.validate_candidate(
            [("inverse", Posture::Create, "Number", &old)].into_iter(),
            &lawful,
            &request
        ),
        Err(AuthoredCorrectionCandidateDenial::Identity(_))
    ));
}
#[test]
fn candidate_role_scan_preserves_cancellation_as_interruption() {
    let prepared = prepared(Posture::Create);
    let cancellation = WorthQueryCancellationSource::new();
    let request = WorthQueryRequestScope::new(None, cancellation.token());
    let exact = EntityReference::Existing(prepared.original);
    let roles = [("inverse", Posture::Retire, "Number", &exact)]
        .into_iter()
        .inspect(|_| cancellation.cancel());
    assert!(matches!(
        prepared.validate_candidate(roles, &WorkerIntentBatch::new("retire"), &request),
        Err(AuthoredCorrectionCandidateDenial::Interrupted(
            WorthQueryRequestInterruption::Cancelled
        ))
    ));
}
