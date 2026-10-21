//! Descriptive identity of an owner-issued native observation, never its proof.

use super::*;
use crate::domain_computation::primary_graph::output_lineage::invalidation::ObservedSourcePublication;

pub(super) fn append(
    entries: &mut Vec<CanonicalBasisEntry>,
    prefix: &str,
    observed: &ObservedSourcePublication,
) {
    kind(entries, prefix, "source-publication");
    push(entries, format!("{prefix}.posture"), CanonicalBasisEntryKind::Identity, text(match observed.posture() {
        crate::domain_computation::primary_graph::invariant_projection::ObservedOutputPosture::Current => "current",
        crate::domain_computation::primary_graph::invariant_projection::ObservedOutputPosture::Pending => "pending",
    }));
    let address = observed.address();
    let schema = address.schema();
    let scope = address.scope();
    let occurrence = address.occurrence();
    for (name, value) in [
        ("runtime", address.runtime()),
        ("schema-runtime", schema.runtime_ordinal()),
        ("schema-generation", schema.generation()),
        ("occurrence-owner", occurrence.owner_identity().get()),
        ("occurrence", occurrence.ordinal()),
        ("scope-partition", u64::from(scope.partition_id())),
        ("scope-slot", scope.local_slot()),
        ("scope-generation", u64::from(scope.generation())),
    ] {
        number_u64(entries, prefix, name, value);
    }
    for (name, bytes) in [
        ("package", *schema.package_identity()),
        ("schema", *schema.schema_identity()),
        ("partition", CanonicalDigestId::new(address.partition())),
        (
            "source-epoch",
            CanonicalDigestId::new(observed.source_epoch()),
        ),
    ] {
        push(
            entries,
            format!("{prefix}.{name}"),
            CanonicalBasisEntryKind::Identity,
            CanonicalBasisValue::BytesDigest(bytes),
        );
    }
    push(
        entries,
        format!("{prefix}.family"),
        CanonicalBasisEntryKind::Identity,
        text(address.family().as_str()),
    );
    let heads = observed.completed_heads();
    number(entries, prefix, "heads", heads.len());
    for (index, (binding, partition, occurrence, generation, slot)) in heads.enumerate() {
        let prefix = format!("{prefix}.head.{index}");
        push(
            entries,
            format!("{prefix}.binding"),
            CanonicalBasisEntryKind::Identity,
            text(binding),
        );
        push(
            entries,
            format!("{prefix}.partition"),
            CanonicalBasisEntryKind::Identity,
            CanonicalBasisValue::BytesDigest(CanonicalDigestId::new(partition)),
        );
        number_u64(entries, &prefix, "owner", occurrence.owner_identity().get());
        number_u64(entries, &prefix, "occurrence", occurrence.ordinal());
        number_u64(entries, &prefix, "generation", generation);
        number(entries, &prefix, "slot", slot);
    }
}
