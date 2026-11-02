//! Admitted reads certify stored bytes through the production re-encoding check.
//! Literal encodings are independent oracles; these are not raw-source observations.
use super::*;
use worth_query_host::facade::{
    declaration::application_schema::ApplicationScalarValueBinding,
    runtime::ExecutionAllocationPolicy,
};

pub(super) struct CertifiedObservation {
    identity: WorthQueryInvariantEntityIdentity<NumericSchema, Number>,
    snapshot: WorthQueryApplicationInvariantProjectionSnapshot<NumericSchema>,
}
impl CertifiedObservation {
    pub(super) fn capture(
        application: &host::Host,
        authority: &WorthQueryApplicationInvariantProjectionAuthority<NumericSchema>,
        key: &str,
    ) -> Self {
        let scope = request_scope();
        let projected = application
            .runtime()
            .with_application_advancement(&scope, |phase| {
                authority.project(
                    &phase,
                    |reader| {
                        reader
                            .resolve_entity(Key::reference(), key.to_owned())
                            .unwrap()
                    },
                    ExecutionAllocationPolicy::SystemAllocation,
                )
            })
            .unwrap()
            .unwrap();
        let (identity, snapshot, _) = projected.into_parts();
        Self { identity, snapshot }
    }
    pub(super) fn primary(&self, application: &host::Host) -> AspectValue {
        let runtime = application.runtime();
        let scope = request_scope();
        let value = runtime
            .with_application_advancement(&scope, |phase| {
                let execution = phase
                    .checked_execution_request_for(runtime.product_runtime())
                    .unwrap();
                self.snapshot
                    .read_fields(&execution, |reader| {
                        reader.field(&self.identity, Primary::reference())
                    })
                    .unwrap()
                    .unwrap()
                    .unwrap()
            })
            .unwrap();
        InlineCounterBinding::encode(&value).unwrap()
    }
    pub(super) fn revision(&self, application: &host::Host) -> u64 {
        use super::super::model::composite_definition_model::DefinitionRevision;
        let runtime = application.runtime();
        let scope = request_scope();
        runtime
            .with_application_advancement(&scope, |phase| {
                let execution = phase
                    .checked_execution_request_for(runtime.product_runtime())
                    .unwrap();
                self.snapshot
                    .read_fields(&execution, |reader| {
                        reader.field(&self.identity, DefinitionRevision::reference())
                    })
                    .unwrap()
                    .unwrap()
                    .unwrap()
            })
            .unwrap()
    }
}

/// Recovered Hosts expose ordinary admitted reads, with the same binary decoder.
pub(super) fn current(application: &host::Host, key: &str) -> AspectValue {
    let scope = request_scope();
    let principal = authenticate_operator(application.installed_schema(), &scope);
    let result = application
        .runtime()
        .request(&principal, &scope)
        .query(super::super::model::query::Read {
            key: key.to_owned(),
        })
        .execute()
        .unwrap();
    assert_eq!(result.rows().len(), 1);
    InlineCounterBinding::encode(&result.rows()[0].primary).unwrap()
}

/// Exact whole-model equality detects any effect from an admitted read.
/// Cross-runtime identity is checked by literal payloads, not rebinding metadata.
pub(super) fn stable_current(application: &host::Host, key: &str) -> AspectValue {
    let before = capture(application);
    let value = current(application, key);
    let after = capture(application);
    application
        .with_host_advancement(|phase| {
            let first = application
                .runtime()
                .inspect_application_checkpoint(&phase, before.bytes())
                .unwrap();
            let second = application
                .runtime()
                .inspect_application_checkpoint(&phase, after.bytes())
                .unwrap();
            assert!(
                first.model == second.model,
                "an admitted read must preserve complete model bytes"
            );
        })
        .unwrap();
    value
}

pub(super) fn assert_wire(value: &AspectValue, expected: &[u8; 9]) {
    let AspectValue::InlineBytes(bytes) = value else {
        panic!("native field must carry inline bytes: {value:?}")
    };
    assert_eq!(bytes.as_bytes(), expected);
}

pub(super) fn capture(application: &host::Host) -> WorthQueryApplicationCheckpoint {
    application
        .with_host_advancement(|phase| {
            application
                .capture_application_checkpoint(
                    &phase,
                    WorthQueryCheckpointCapturePolicy::SystemAllocation,
                )
                .unwrap()
        })
        .unwrap()
}

/// Real Foundational phase progression; byte literals remain the separate oracle.
pub(super) fn digest(value: &AspectValue) -> [u8; 32] {
    use worth_foundational::facade::{
        canonical_basis_value_for_aspect_value, canonicalization, derive_canonical_digest,
        prepare_canonical_basis_sequence, CanonicalBasisDomain, CanonicalBasisEntry,
        CanonicalBasisEntryKind, CanonicalBasisLocus, CanonicalDigestAlgorithmId,
        CanonicalizationRuleVersion,
    };
    let sequence = prepare_canonical_basis_sequence(
        CanonicalizationRuleVersion::new("fixture.inline-counter.digest.v1").unwrap(),
        CanonicalBasisDomain::Value,
        [CanonicalBasisEntry::new(
            CanonicalBasisDomain::Value,
            CanonicalBasisLocus::Root,
            CanonicalBasisEntryKind::Value,
            canonical_basis_value_for_aspect_value(value),
        )],
    )
    .unwrap();
    let ready = canonicalization()
        .digest()
        .for_sequence(sequence, CanonicalDigestAlgorithmId::sha256())
        .unwrap();
    *derive_canonical_digest(ready).value().bytes()
}
