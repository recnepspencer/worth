//! Aggregate identity minting.
//!
//! [`mint`] is the legacy v1 native-member domain still validated for stored
//! aggregate rows. [`mint_group_name`] is the v2 allocation-independent group
//! name domain: it hashes only `(scheme, anchor name, continuity rule)` and
//! never a `CommitId` or native `(partition, local_slot, generation)`.
use super::group::{AggregateContinuityRule, AggregateGroupSchemeId};
use crate::graph_member::NativeMemberKey as Member;
use crate::history::data::CommitId;
use crate::lineage::data::PersistentNameIdentity;
use sha2::{Digest, Sha256};
use worth_foundational::facade::{AggregateDefinitionIdentity, AggregateIdentity};

/// Explicit v2 domain separator: v1 bytes are never reinterpreted as v2 names.
pub const GROUP_AGGREGATE_IDENTITY_DOMAIN_TAG: &[u8] =
    b"worth.relational.group-aggregate-identity.v2\0";

pub(in crate::aggregate_identity) fn mint_group_name(
    scheme: &AggregateGroupSchemeId,
    anchor: PersistentNameIdentity,
    rule: AggregateContinuityRule,
) -> AggregateIdentity {
    let mut hash = Sha256::new();
    hash.update(GROUP_AGGREGATE_IDENTITY_DOMAIN_TAG);
    text(&mut hash, scheme.as_str());
    hash.update(anchor.as_bytes());
    hash.update([rule.canonical_tag()]);
    AggregateIdentity::from_bytes(hash.finalize().into())
}

pub(in crate::aggregate_identity) fn mint(
    definition: AggregateDefinitionIdentity,
    occurrence: CommitId,
    anchor: impl Into<Member>,
) -> AggregateIdentity {
    let anchor = anchor.into();
    let mut hash = Sha256::new();
    match &anchor {
        Member::Entity(_) => hash.update(b"WORTH_GRAPH_AGGREGATE_ID_V1\0"),
        Member::Projected(_) => hash.update(b"WORTH_PROJECTED_GRAPH_AGGREGATE_ID_V1\0"),
    }
    hash.update(definition.bytes());
    hash.update(occurrence.0.to_be_bytes());
    if let Member::Projected(member) = &anchor {
        text(&mut hash, &member.domain.schema.0);
        hash.update(member.domain.version.0.to_be_bytes());
        text(&mut hash, &member.domain.name);
    }
    let carrier = anchor.carrier();
    hash.update(carrier.partition_value_u64().to_be_bytes());
    hash.update(carrier.local_slot_value().to_be_bytes());
    hash.update(carrier.generation_value().to_be_bytes());
    if let Member::Projected(member) = &anchor {
        text(&mut hash, &member.case);
    }
    AggregateIdentity::from_bytes(hash.finalize().into())
}
fn text(hash: &mut Sha256, text: &str) {
    hash.update((text.len() as u64).to_be_bytes());
    hash.update(text.as_bytes());
}
