use worth_foundational::facade::{
    canonical_basis_value_for_aspect_value, canonicalization, prepare_canonical_basis_sequence,
    CanonicalBasisDomain, CanonicalBasisEntry, CanonicalBasisEntryKind, CanonicalBasisLocus,
    CanonicalBasisValue, CanonicalDigestAlgorithmId, CanonicalDigestId, CanonicalDigestWorkBudget,
    CanonicalizationRuleVersion,
};
use worth_query_declaration::facade::application_schema::TypedMutationPrecondition;
use worth_query_installation::facade::WorthQueryCanonicalWorkEvidence;
use worth_relational::facade::identity::EntityId;

const DOMAIN: CanonicalBasisDomain =
    CanonicalBasisDomain::Future("worth-query.mutation-precondition-binding");
const RULE_VERSION: &str = "worth-query-mutation-precondition-binding-v1";

pub(super) struct WorthQueryPreconditionCanonicalIdentity {
    pub digest: CanonicalDigestId,
    pub work: WorthQueryCanonicalWorkEvidence,
}

pub(super) fn prepare_precondition_identity(
    entries: &[&TypedMutationPrecondition],
    scope_entity_id: EntityId,
    budget: CanonicalDigestWorkBudget,
) -> Result<WorthQueryPreconditionCanonicalIdentity, ()> {
    let mut basis_entries = Vec::with_capacity(entries.len().saturating_mul(5).saturating_add(1));
    basis_entries.push(entry(
        "scope.entity-id",
        CanonicalBasisEntryKind::Identity,
        CanonicalBasisValue::EntityRef {
            partition_id: scope_entity_id.partition_id.0,
            local_slot: scope_entity_id.local_slot.0,
            generation: scope_entity_id.generation.0,
        },
    ));
    for (index, precondition) in entries.iter().enumerate() {
        let target = precondition.target();
        basis_entries.push(text(index, "family", target.family().canonical_name()));
        basis_entries.push(text(index, "entity", target.entity()));
        basis_entries.push(text(index, "aspect", target.aspect()));
        basis_entries.push(text(index, "field", target.field_name()));
        basis_entries.push(entry(
            format!("precondition.{index}.value"),
            CanonicalBasisEntryKind::Value,
            canonical_basis_value_for_aspect_value(precondition.expected_value()),
        ));
    }

    let version = CanonicalizationRuleVersion::new(RULE_VERSION).ok_or(())?;
    let basis = prepare_canonical_basis_sequence(version, DOMAIN, basis_entries)
        .into_result()
        .map_err(|_| ())?;
    let digest_ready = canonicalization()
        .digest()
        .for_sequence_with_budget(basis, CanonicalDigestAlgorithmId::sha256(), budget)
        .into_result()
        .map_err(|_| ())?;
    let derived = canonicalization().digest().derive(digest_ready);
    Ok(WorthQueryPreconditionCanonicalIdentity {
        digest: CanonicalDigestId::new(*derived.value().bytes()),
        work: WorthQueryCanonicalWorkEvidence::one_digest(derived.metadata().work()),
    })
}

fn text(index: usize, field: &str, value: &str) -> CanonicalBasisEntry {
    entry(
        format!("precondition.{index}.{field}"),
        CanonicalBasisEntryKind::Locator,
        CanonicalBasisValue::ExactText(value.to_owned().into()),
    )
}

fn entry(
    locus: impl Into<String>,
    kind: CanonicalBasisEntryKind,
    value: CanonicalBasisValue,
) -> CanonicalBasisEntry {
    CanonicalBasisEntry::new(
        DOMAIN,
        CanonicalBasisLocus::Named(locus.into().into()),
        kind,
        value,
    )
}
