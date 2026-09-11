//! Q8.25-C2: the exact outbox create-intent projection, field by field.
//!
//! The path under test is the whole production chain, not a stub of it: an
//! admitted emission batch projects its external payload, the provider derives
//! the outbox record from the *installed* contract, and the record lowers into
//! the create intent that will co-commit with the mutation. This is projection
//! evidence only; a fresh-owner read of a committed row belongs to F8.6.
//!
//! The point is that nothing in that chain accepts a caller-supplied effect
//! name, payload type, payload bytes, or byte bound. Each of those four is
//! asserted against the installed contract or the admitted emission it came
//! from, never against a value the test also handed to the derivation.

use worth_foundational::facade::{
    AspectValue, BoundaryProtocolIdentity, BoundaryProtocolVersion, InternedString,
};
use worth_query_declaration::facade::application_schema::{
    ApplicationExternalEffectBinding, ApplicationExternalEffectProtocol,
    ApplicationRetainedEffectBinding,
};
use worth_query_installation::facade::InstalledExternalEffectContract;
use worth_relational::facade::history::BranchId;
use worth_relational::facade::identity::KindId;
use worth_relational::facade::transactions::{CreateIntent, EntitySpec, MutationIntent};

use crate::domain_computation::application_aftermath::{
    dispatch_outbox_create_intent, WorthQueryDispatchOutboxLayout, WorthQueryDispatchOutboxRecord,
};
use crate::domain_computation::primary_graph::application_attempt::{
    WorthQueryApplicationCommitOutcomeIdentity, WorthQueryApplicationIdempotencyBinding,
};
use crate::domain_computation::primary_graph::provider::dispatch_outbox::{
    derive_dispatch_outbox_record, WorthQueryDispatchOutboxBasis,
};
use crate::domain_computation::primary_graph::schema_layout::planned_field_locator;

use super::model::{WorthQueryAdmittedApplicationEmissionBatch, WorthQueryApplicationEmission};

const EFFECT: &str = "DeathNoticeEffect";
const FAMILY: &str = "estate-death-notice-rail";
const NOTICE: &[u8] = b"decease";
const EXTERNAL_PROTOCOL: ApplicationExternalEffectProtocol = ApplicationExternalEffectProtocol::new(
    BoundaryProtocolIdentity::new("test.death-notice"),
    BoundaryProtocolVersion::new(1),
);

#[derive(Clone)]
struct DeathNotice(Vec<u8>);
worth_query_declaration::worth_query_portable_type!(
    DeathNotice => "worth.query.test.death-notice.v1"
);

worth_query_declaration::worth_query_structured_value_binding!(DeathNoticeBinding for DeathNotice { identity: "worth.query.test.death-notice.v1" });
impl ApplicationRetainedEffectBinding for DeathNoticeBinding {
    fn retained_bytes(value: &Self::Value) -> u64 {
        u64::try_from(std::mem::size_of::<Self::Value>() + value.0.capacity()).unwrap_or(u64::MAX)
    }
}

impl ApplicationExternalEffectBinding for DeathNoticeBinding {
    const PROTOCOL: ApplicationExternalEffectProtocol = EXTERNAL_PROTOCOL;
    const MAX_EXTERNAL_BYTES: u64 = 8;

    fn external_effect_bytes(value: &Self::Value) -> Vec<u8> {
        value.0.clone()
    }
}

struct OutboxProjectionEvidence {
    spec: EntitySpec,
    layout: WorthQueryDispatchOutboxLayout,
    record: WorthQueryDispatchOutboxRecord,
    outcome: WorthQueryApplicationCommitOutcomeIdentity,
}

#[test]
fn outbox_create_intent_projects_the_contract_and_admitted_payload_exactly() {
    let evidence = declared_outbox_projection();
    assert_protocol_projection(&evidence);
    assert_correlation_projection(&evidence);
}

fn declared_outbox_projection() -> OutboxProjectionEvidence {
    let contract = installed_contract();
    let batch = admitted_notice();
    let projected = batch
        .external_payload(&contract)
        .expect("the admitted batch projects its external payload")
        .expect("a declared external effect projects some payload");

    let outcome = WorthQueryApplicationCommitOutcomeIdentity::mint().expect("outcome identity");
    let record = derive_dispatch_outbox_record(WorthQueryDispatchOutboxBasis {
        external_effect: &contract,
        external_payload: Some(&projected),
        operation_slot: "notify-death",
        operation_version: 1,
        idempotency: WorthQueryApplicationIdempotencyBinding::new([0xAB; 32], [0xCD; 32]),
        outcome_identity: outcome,
        branch: &BranchId("main".to_owned()),
    })
    .expect("a declared external effect derives its outbox record")
    .expect("a declared external effect derives some record");

    let layout = layout();
    let Some(MutationIntent::Create(CreateIntent::Entity(spec))) =
        dispatch_outbox_create_intent(Some(&layout), Some(&record))
    else {
        panic!("a declared external effect co-commits an outbox entity");
    };

    OutboxProjectionEvidence {
        spec,
        layout,
        record,
        outcome,
    }
}

fn assert_protocol_projection(evidence: &OutboxProjectionEvidence) {
    let spec = &evidence.spec;
    let layout = &evidence.layout;
    assert_eq!(spec.kind_id, layout.entity_kind);
    assert_eq!(
        spec.fields.len(),
        8,
        "the outbox projects exactly its eight declared fields"
    );
    assert_eq!(
        spec.fields.get(&layout.payload_locator),
        Some(&text(hex(DeathNoticeBinding::external_effect_bytes(
            &DeathNotice(NOTICE.to_vec()),
        )))),
        "the persisted payload is the emission's own wire projection"
    );
    assert_eq!(
        spec.fields.get(&layout.effect_locator),
        Some(&text(EFFECT.to_owned())),
        "the persisted effect name comes from the installed contract"
    );
    assert_eq!(
        spec.fields.get(&layout.protocol_identity_locator),
        Some(&text(EXTERNAL_PROTOCOL.identity().as_str().to_owned())),
        "the persisted protocol identity comes from the installed contract"
    );
    assert_eq!(
        spec.fields.get(&layout.protocol_version_locator),
        Some(&AspectValue::UInt64(u64::from(
            EXTERNAL_PROTOCOL.version().get(),
        ))),
        "the persisted exact version comes from the installed contract"
    );
    assert_eq!(
        spec.fields.get(&layout.maximum_payload_bytes_locator),
        Some(&AspectValue::UInt64(DeathNoticeBinding::MAX_EXTERNAL_BYTES)),
        "the persisted bound is the payload type's own associated constant"
    );
    assert_eq!(
        spec.fields.get(&layout.family_locator),
        Some(&text(FAMILY.to_owned())),
        "the persisted rail comes from the installed contract"
    );
}

fn assert_correlation_projection(evidence: &OutboxProjectionEvidence) {
    let spec = &evidence.spec;
    let layout = &evidence.layout;
    assert_eq!(
        spec.fields.get(&layout.outcome_identity_locator),
        Some(&AspectValue::UInt64(evidence.outcome.get())),
        "the persisted outcome identity is the one this commit minted"
    );
    let correlation = hex(evidence.record.correlation().digest().bytes().to_vec());
    assert_eq!(
        spec.fields.get(&layout.correlation_locator),
        Some(&text(correlation.clone())),
        "the persisted correlation is the derived one"
    );
    assert_eq!(
        spec.client_key.as_raw_str(),
        Some(format!("worth-query-dispatch-outbox:{correlation}").as_str()),
        "the outbox row is keyed by its correlation, so a redelivery converges"
    );
}

/// The record cannot be derived at all without an admitted matching emission.
///
/// This is the arm that makes the payload non-optional in practice: a declared
/// lane with nothing projected onto it denies rather than co-committing an empty
/// or caller-chosen payload.
#[test]
fn a_declared_lane_with_no_projected_payload_derives_nothing() {
    let failure = derive_dispatch_outbox_record(WorthQueryDispatchOutboxBasis {
        external_effect: &installed_contract(),
        external_payload: None,
        operation_slot: "notify-death",
        operation_version: 1,
        idempotency: WorthQueryApplicationIdempotencyBinding::new([0xAB; 32], [0xCD; 32]),
        outcome_identity: WorthQueryApplicationCommitOutcomeIdentity::mint()
            .expect("outcome identity"),
        branch: &BranchId("main".to_owned()),
    })
    .expect_err("a declared lane cannot commit without its projected payload");
    assert!(matches!(
        failure,
        crate::domain_computation::application_aftermath::WorthQueryAftermathDerivationFailure::MissingExternalPayload
    ));
}

/// R8.4: an operation that declares no lane pays for no outbox row.
#[test]
fn an_undeclared_lane_derives_and_persists_nothing() {
    let record = derive_dispatch_outbox_record(WorthQueryDispatchOutboxBasis {
        external_effect: &InstalledExternalEffectContract::None,
        external_payload: Some(NOTICE),
        operation_slot: "release-estate",
        operation_version: 1,
        idempotency: WorthQueryApplicationIdempotencyBinding::new([0xAB; 32], [0xCD; 32]),
        outcome_identity: WorthQueryApplicationCommitOutcomeIdentity::mint()
            .expect("outcome identity"),
        branch: &BranchId("main".to_owned()),
    })
    .expect("an undeclared lane derives without failing");
    assert!(
        record.is_none(),
        "an undeclared lane derives no record even when a payload is at hand"
    );
    assert!(dispatch_outbox_create_intent(Some(&layout()), record.as_ref()).is_none());
}

fn installed_contract() -> InstalledExternalEffectContract {
    InstalledExternalEffectContract::Declared {
        correlation_family:
            worth_query_installation::facade::WorthQueryExternalEffectCorrelationFamily::new(FAMILY)
                .unwrap(),
        effect: EFFECT.to_owned(),
        rust_payload_type: <DeathNotice as worth_query_declaration::facade::portable_identity::WorthQueryPortableType>::PORTABLE_TYPE_IDENTITY,
        protocol: EXTERNAL_PROTOCOL,
        maximum_payload_bytes: DeathNoticeBinding::MAX_EXTERNAL_BYTES,
    }
}

fn admitted_notice() -> WorthQueryAdmittedApplicationEmissionBatch {
    WorthQueryAdmittedApplicationEmissionBatch::admit(
        vec![
            WorthQueryApplicationEmission::new_external::<DeathNoticeBinding>(
                EFFECT,
                DeathNotice(NOTICE.to_vec()),
            )
            .expect("the fixture projection stays within its declared bound"),
        ],
        1_024,
    )
    .expect("the fixture batch stays within its retained-byte ceiling")
}

fn layout() -> WorthQueryDispatchOutboxLayout {
    let locator = |field: &str| {
        planned_field_locator("dispatch-outbox", field).expect("fixture locator is well-formed")
    };
    WorthQueryDispatchOutboxLayout {
        entity_kind: KindId::new(9_001),
        correlation_locator: locator("correlation"),
        family_locator: locator("correlation-family"),
        effect_locator: locator("effect"),
        protocol_identity_locator: locator("protocol-identity"),
        protocol_version_locator: locator("protocol-version"),
        maximum_payload_bytes_locator: locator("maximum-payload-bytes"),
        payload_locator: locator("payload-hex"),
        outcome_identity_locator: locator("outcome-identity"),
    }
}

fn text(value: String) -> AspectValue {
    AspectValue::String(InternedString::from(value))
}

fn hex(bytes: Vec<u8>) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}
