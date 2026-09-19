//! The client keys a footprint's created-record loci carry.
//!
//! A locus naming a record this transaction creates carries the author's own
//! client key, which starts life as a raw string and must end as an interned
//! symbol. Both halves of that passage live here: harvesting the raw strings a
//! footprint still holds, so the interner can be asked for symbols once, and
//! rewriting the loci onto those symbols afterwards. Keeping them together is
//! what makes the two sides agree about which loci carry a key at all.

use std::collections::BTreeSet;

use crate::symbols::data::{ClientKeySymbolPolicy, StringInterner};
use crate::transactions::data::{CreatedEntityRef, CreatedRelationRef, EntityReference};

use super::{RelationalTransactionReadLocus, RelationalTransactionWriteLocus};

pub(super) fn collect_created_entity_raw_key(
    created: &CreatedEntityRef,
    raw_values: &mut BTreeSet<String>,
) {
    if let Some(raw) = created.client_key.as_raw_str() {
        raw_values.insert(raw.to_owned());
    }
}

pub(super) fn collect_created_relation_raw_keys(
    created: &CreatedRelationRef,
    raw_values: &mut BTreeSet<String>,
) {
    if let Some(raw) = created.client_key.as_raw_str() {
        raw_values.insert(raw.to_owned());
    }
    collect_entity_reference_raw_key(&created.source, raw_values);
    collect_entity_reference_raw_key(&created.target, raw_values);
}

pub(super) fn collect_entity_reference_raw_key(
    reference: &EntityReference,
    raw_values: &mut BTreeSet<String>,
) {
    if let EntityReference::Created(created) = reference {
        collect_created_entity_raw_key(created, raw_values);
    }
}

pub(super) fn normalize_read_locus(
    read: RelationalTransactionReadLocus,
    interner: &mut StringInterner,
    policy: ClientKeySymbolPolicy,
) -> RelationalTransactionReadLocus {
    match read {
        RelationalTransactionReadLocus::CreatedEntity(created) => {
            RelationalTransactionReadLocus::CreatedEntity(normalize_created_entity(
                created, interner, policy,
            ))
        }
        RelationalTransactionReadLocus::CreatedRelation(created) => {
            RelationalTransactionReadLocus::CreatedRelation(normalize_created_relation(
                created, interner, policy,
            ))
        }
        other => other,
    }
}

pub(super) fn normalize_write_locus(
    write: RelationalTransactionWriteLocus,
    interner: &mut StringInterner,
    policy: ClientKeySymbolPolicy,
) -> RelationalTransactionWriteLocus {
    match write {
        RelationalTransactionWriteLocus::CreatedEntity(created) => {
            RelationalTransactionWriteLocus::CreatedEntity(normalize_created_entity(
                created, interner, policy,
            ))
        }
        RelationalTransactionWriteLocus::CreatedRelation(created) => {
            RelationalTransactionWriteLocus::CreatedRelation(normalize_created_relation(
                created, interner, policy,
            ))
        }
        other => other,
    }
}

pub(super) fn normalize_created_entity(
    mut created: CreatedEntityRef,
    interner: &mut StringInterner,
    policy: ClientKeySymbolPolicy,
) -> CreatedEntityRef {
    created.client_key = created.client_key.normalize_with(interner, policy);
    created
}

pub(super) fn normalize_created_relation(
    mut created: CreatedRelationRef,
    interner: &mut StringInterner,
    policy: ClientKeySymbolPolicy,
) -> CreatedRelationRef {
    created.client_key = created.client_key.normalize_with(interner, policy);
    created.source = normalize_entity_reference(created.source, interner, policy);
    created.target = normalize_entity_reference(created.target, interner, policy);
    created
}

pub(super) fn normalize_entity_reference(
    reference: EntityReference,
    interner: &mut StringInterner,
    policy: ClientKeySymbolPolicy,
) -> EntityReference {
    match reference {
        EntityReference::Created(created) => {
            EntityReference::Created(normalize_created_entity(created, interner, policy))
        }
        existing => existing,
    }
}
