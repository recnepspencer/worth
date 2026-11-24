//! Aggregate identity minting.
//!
//! [`mint`] is the legacy v1 native-member domain still validated for stored
//! aggregate rows. The v2 group domains hash only persistent names and never a
//! `CommitId` or native `(partition, local_slot, generation)`:
//! [`mint_group_name`] names a group from `(scheme, anchor, rule)`, and
//! [`mint_group_transition`] additionally qualifies an unkept split or merge
//! successor by the sorted identities of the live prior groups it received
//! members from.
use super::group::{AggregateContinuityRule, AggregateGroupSchemeId};
use crate::contracts::graph_member::key::NativeMemberKey as Member;
use crate::contracts::history::data::coordinates::CommitId;
use crate::contracts::lineage::data::naming::PersistentNameIdentity;
use sha2::{Digest, Sha256};
use worth_foundational::facade::{AggregateDefinitionIdentity, AggregateIdentity};

/// Explicit v2 domain separator: v1 bytes are never reinterpreted as v2 names.
pub const GROUP_AGGREGATE_IDENTITY_DOMAIN_TAG: &[u8] =
    b"worth.relational.group-aggregate-identity.v2\0";
/// Distinct v2 domain for unkept split/merge successors with prior sources.
pub const GROUP_AGGREGATE_TRANSITION_DOMAIN_TAG: &[u8] =
    b"worth.relational.group-aggregate-transition.v2\0";

pub(in crate::internal_port::aggregate_identity) fn mint_group_name(
    scheme: &AggregateGroupSchemeId,
    anchor: PersistentNameIdentity,
    rule: AggregateContinuityRule,
) -> AggregateIdentity {
    let mut hash = Sha256::new();
    hash.update(GROUP_AGGREGATE_IDENTITY_DOMAIN_TAG);
    group_terms(&mut hash, scheme, anchor, rule);
    AggregateIdentity::from_bytes(hash.finalize().into())
}

/// `sources` must be strictly ascending; it enters count-prefixed as fixed
/// 32-byte identities so the hash input stays O(1) per source at any depth.
pub(in crate::internal_port::aggregate_identity) fn mint_group_transition(
    scheme: &AggregateGroupSchemeId,
    anchor: PersistentNameIdentity,
    rule: AggregateContinuityRule,
    sources: &[AggregateIdentity],
) -> AggregateIdentity {
    debug_assert!(sources.windows(2).all(|pair| pair[0] < pair[1]));
    let mut hash = Sha256::new();
    hash.update(GROUP_AGGREGATE_TRANSITION_DOMAIN_TAG);
    group_terms(&mut hash, scheme, anchor, rule);
    hash.update((sources.len() as u64).to_be_bytes());
    for source in sources {
        hash.update(source.bytes());
    }
    AggregateIdentity::from_bytes(hash.finalize().into())
}

/// A pure group name when no live prior group feeds it, else its successor.
pub(in crate::internal_port::aggregate_identity) fn mint_group_identity(
    scheme: &AggregateGroupSchemeId,
    anchor: PersistentNameIdentity,
    rule: AggregateContinuityRule,
    sources: &[AggregateIdentity],
) -> AggregateIdentity {
    if sources.is_empty() {
        mint_group_name(scheme, anchor, rule)
    } else {
        mint_group_transition(scheme, anchor, rule, sources)
    }
}

fn group_terms(
    hash: &mut Sha256,
    scheme: &AggregateGroupSchemeId,
    anchor: PersistentNameIdentity,
    rule: AggregateContinuityRule,
) {
    text(hash, scheme.as_str());
    hash.update(anchor.as_bytes());
    hash.update([rule.canonical_tag()]);
}

pub(in crate::internal_port::aggregate_identity) fn mint(
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
