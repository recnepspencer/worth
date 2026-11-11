//! Canonical identities for each stage of the external-effect causal ladder.

use worth_foundational::facade::{
    canonicalization, prepare_canonical_basis_sequence, CanonicalBasisDomain, CanonicalBasisEntry,
    CanonicalBasisEntryKind, CanonicalBasisLocus, CanonicalBasisValue, CanonicalDigestAlgorithmId,
    CanonicalDigestId, CanonicalDigestWorkBudget, CanonicalIntegerWidth,
    CanonicalizationRuleVersion,
};
use worth_query_installation::facade::WorthQueryCanonicalWorkEvidence;

use super::super::WorthQueryAftermathDerivationFailure;
use super::identity::ExternalEffectPostureIdentity;
use super::ExternalEffectCorrelationIdentity;

const DOMAIN: CanonicalBasisDomain =
    CanonicalBasisDomain::Future("worth-query.external-effect-causal-event");
const RULE_VERSION: &str = "worth-query-external-effect-causal-event-v5";
const BUDGET: CanonicalDigestWorkBudget = match CanonicalDigestWorkBudget::new(18, 4 * 1_024) {
    Some(budget) => budget,
    None => panic!("fixed external-effect causal event budget is valid"),
};

pub(super) fn provider_commit_identity(
    runtime: crate::domain_computation::execution_runtime::WorthQueryRuntimeAuthorityIdentity,
    observation: &crate::domain_computation::primary_graph::WorthQueryCommittedDispatchOutboxObservation,
) -> Result<DerivedEventIdentity, WorthQueryAftermathDerivationFailure> {
    let commit = observation.commit_reference();
    let product = observation.committed_product_publication();
    let (record_kind, partition, slot, generation) = match observation.record_ref() {
        worth_relational::facade::transactions::RecordRef::Entity(record) => (
            "entity",
            record.partition_value_u64(),
            record.local_slot_value(),
            u64::from(record.generation_value()),
        ),
        worth_relational::facade::transactions::RecordRef::Relation(record) => (
            "relation",
            record.partition_value_u64(),
            record.local_slot_value(),
            u64::from(record.generation_value()),
        ),
    };
    let mut entries = vec![text_entry("stage", "provider-commit")];
    match product.product_branch().owner_identity().durable() {
        Some(owner) => {
            entries.push(text_entry("owner-scope", "home"));
            append_home(&mut entries, owner.home());
            entries.push(integer_entry("world-slot", owner.slot().get()));
        }
        None => {
            entries.push(text_entry("owner-scope", "memory"));
            entries.push(integer_entry("query-runtime", runtime.as_u64()));
            entries.push(integer_entry(
                "world-owner",
                product.product_branch().owner_identity().get(),
            ));
        }
    }
    entries.extend([
        text_entry("product-branch", product.product_branch().name().as_str()),
        integer_entry(
            "product-incarnation",
            product.product_incarnation().ordinal(),
        ),
        integer_entry("product-generation", product.product_generation().get()),
        integer_entry("composite-commit", product.composite_commit().ordinal()),
        integer_entry(
            "publication-attempt",
            product.publication_attempt().ordinal(),
        ),
        digest_entry("correlation", observation.record().correlation().digest()),
        text_entry("record-kind", record_kind),
        integer_entry("record-partition", partition),
        integer_entry("record-slot", slot),
        integer_entry("record-generation", generation),
        text_entry("branch", &commit.branch_id.0),
        integer_entry("commit", commit.commit_id.0),
        integer_entry("version", commit.version_id.0),
    ]);
    derive(entries)
}

pub(super) fn emission_identity(
    predecessor: &ExternalEffectPostureIdentity,
    correlation: &ExternalEffectCorrelationIdentity,
    outcome_identity: u64,
) -> Result<DerivedEventIdentity, WorthQueryAftermathDerivationFailure> {
    derive(vec![
        text_entry("stage", "co-committed-application-emission"),
        digest_entry("predecessor", predecessor.digest()),
        digest_entry("correlation", correlation.digest()),
        integer_entry("outcome", outcome_identity),
    ])
}

pub(super) fn attempt_identity(
    predecessor: &ExternalEffectPostureIdentity,
    correlation: &ExternalEffectCorrelationIdentity,
    coordinate: &super::dispatch_admission::DispatchAttemptCoordinate<'_>,
) -> Result<DerivedEventIdentity, WorthQueryAftermathDerivationFailure> {
    let mut entries = vec![
        text_entry("stage", "dispatch-attempt"),
        digest_entry("predecessor", predecessor.digest()),
        digest_entry("correlation", correlation.digest()),
    ];
    match coordinate {
        super::dispatch_admission::DispatchAttemptCoordinate::Memory(ordinal) => {
            entries.push(text_entry("attempt-scope", "memory"));
            entries.push(integer_entry("attempt-ordinal", ordinal.value()));
        }
        super::dispatch_admission::DispatchAttemptCoordinate::Recorded(contact) => {
            let recorded = contact.recorded_attempt();
            entries.push(text_entry("attempt-scope", "home-journal"));
            append_home(&mut entries, recorded.home());
            entries.push(integer_entry(
                "attempt-position",
                recorded.attempt_position().get(),
            ));
        }
    }
    derive(entries)
}

fn append_home(
    entries: &mut Vec<CanonicalBasisEntry>,
    home: worth_foundational::facade::HomeIdentity,
) {
    let bytes = home.bytes();
    entries.push(integer_entry(
        "home-high",
        u64::from_be_bytes(bytes[..8].try_into().unwrap()),
    ));
    entries.push(integer_entry(
        "home-low",
        u64::from_be_bytes(bytes[8..].try_into().unwrap()),
    ));
}

pub(super) fn observation_identity(
    predecessor: &ExternalEffectPostureIdentity,
    correlation: &ExternalEffectCorrelationIdentity,
    observation: &'static str,
) -> Result<DerivedEventIdentity, WorthQueryAftermathDerivationFailure> {
    derive(vec![
        text_entry("stage", "external-owner-observation"),
        digest_entry("predecessor", predecessor.digest()),
        digest_entry("correlation", correlation.digest()),
        text_entry("observation", observation),
    ])
}

pub(super) struct DerivedEventIdentity {
    pub digest: CanonicalDigestId,
    pub work: WorthQueryCanonicalWorkEvidence,
}

fn derive(
    entries: Vec<CanonicalBasisEntry>,
) -> Result<DerivedEventIdentity, WorthQueryAftermathDerivationFailure> {
    let version = CanonicalizationRuleVersion::new(RULE_VERSION)
        .expect("the external-effect causal-event rule is valid");
    let prepared = prepare_canonical_basis_sequence(version, DOMAIN, entries)
        .into_result()
        .map_err(|_| WorthQueryAftermathDerivationFailure::BasisRejected)?;
    let ready = canonicalization()
        .digest()
        .for_sequence_with_budget(prepared, CanonicalDigestAlgorithmId::sha256(), BUDGET)
        .into_result()
        .map_err(|_| WorthQueryAftermathDerivationFailure::DigestRejected)?;
    let derived = canonicalization().digest().derive(ready);
    Ok(DerivedEventIdentity {
        digest: CanonicalDigestId::new(*derived.value().bytes()),
        work: WorthQueryCanonicalWorkEvidence::one_digest(derived.metadata().work()),
    })
}

fn text_entry(locus: &str, value: &str) -> CanonicalBasisEntry {
    entry(
        locus,
        CanonicalBasisValue::ExactText(value.to_owned().into()),
    )
}

fn digest_entry(locus: &str, value: &CanonicalDigestId) -> CanonicalBasisEntry {
    entry(locus, CanonicalBasisValue::BytesDigest(*value))
}

fn integer_entry(locus: &str, value: u64) -> CanonicalBasisEntry {
    entry(
        locus,
        CanonicalBasisValue::UnsignedInteger {
            width: CanonicalIntegerWidth::Bits64,
            value: value.into(),
        },
    )
}

fn entry(locus: &str, value: CanonicalBasisValue) -> CanonicalBasisEntry {
    CanonicalBasisEntry::new(
        DOMAIN,
        CanonicalBasisLocus::Named(locus.to_owned().into()),
        CanonicalBasisEntryKind::Identity,
        value,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn portable_rule_has_a_frozen_distinct_canonical_corpus() {
        // Independently length-framed and SHA-256 hashed under sha256-canonical-v2.
        let derived = derive(vec![
            text_entry("stage", "v5-corpus"),
            integer_entry("query-runtime", 7),
        ])
        .unwrap();
        assert_eq!(
            derived.digest.bytes(),
            &[
                34, 187, 250, 70, 254, 48, 245, 111, 241, 14, 216, 200, 216, 3, 75, 235, 31, 46,
                237, 125, 114, 60, 203, 212, 188, 39, 248, 44, 200, 255, 102, 41
            ]
        );
    }
}
