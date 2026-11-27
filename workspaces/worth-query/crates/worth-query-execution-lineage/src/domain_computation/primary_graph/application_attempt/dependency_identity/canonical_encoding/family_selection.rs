//! Descriptive identity of a `current_output` family selection: the family
//! it selected from and what it selected, never its proof (public #429).

use super::*;
use crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding;
use crate::domain_computation::primary_graph::output_lineage::family_selection::ObservedFamilySelection;
use worth_query_installation::facade::ApplicationSchemaBindingIdentity;

pub fn append(
    entries: &mut Vec<CanonicalBasisEntry>,
    prefix: &str,
    observed: &ObservedFamilySelection,
) {
    kind(entries, prefix, "family-selection-v1");
    let producer = &observed.producer;
    binding(entries, prefix, producer.runtime_authority, &producer.schema, producer.scope);
    for (name, value) in [("family", producer.family), ("entity-type", producer.entity_identifier)] {
        push(entries, format!("{prefix}.{name}"), CanonicalBasisEntryKind::Identity, text(value));
    }
    number(entries, prefix, "sources", observed.sources.len());
    for (index, source) in observed.sources.iter().enumerate() {
        let prefix = format!("{prefix}.source.{index}");
        binding(entries, &prefix, source.runtime_authority, &source.schema, source.scope);
        push(
            entries,
            format!("{prefix}.output-binding"),
            CanonicalBasisEntryKind::Identity,
            text(source.output_binding.as_str()),
        );
    }
    entities(entries, prefix, "outcome", &observed.outcome);
    entities(entries, prefix, "applied", &observed.applied);
    number(entries, prefix, "replaced", observed.replaced.len());
    for (index, (typed, partition)) in observed.replaced.iter().enumerate() {
        let prefix = format!("{prefix}.replaced.{index}");
        push(
            entries,
            format!("{prefix}.typed"),
            CanonicalBasisEntryKind::Identity,
            text(if typed.is_some() { "typed" } else { "untyped" }),
        );
        push(
            entries,
            format!("{prefix}.partition"),
            CanonicalBasisEntryKind::Identity,
            CanonicalBasisValue::BytesDigest(CanonicalDigestId::new(*partition)),
        );
    }
}

fn binding(
    entries: &mut Vec<CanonicalBasisEntry>,
    prefix: &str,
    runtime: u64,
    schema: &ApplicationSchemaBindingIdentity,
    scope: WorthQueryOperationScopeEntityBinding,
) {
    for (name, value) in [
        ("runtime", runtime),
        ("schema-runtime", schema.runtime_ordinal()),
        ("schema-generation", schema.generation()),
        ("scope-partition", u64::from(scope.partition_id())),
        ("scope-slot", scope.local_slot()),
        ("scope-generation", u64::from(scope.generation())),
    ] {
        number_u64(entries, prefix, name, value);
    }
    for (name, bytes) in [("package", *schema.package_identity()), ("schema", *schema.schema_identity())] {
        push(
            entries,
            format!("{prefix}.{name}"),
            CanonicalBasisEntryKind::Identity,
            CanonicalBasisValue::BytesDigest(bytes),
        );
    }
}
