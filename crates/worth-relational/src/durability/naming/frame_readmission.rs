//! Readmission of one decoded frame into a canonical committed batch.
//!
//! Every birth in the frame is first verified against the one canonical
//! persistent-name hash; only then are references bound, so a name may
//! reference another born earlier in the log or in the same commit, and
//! nothing else. Any refusal leaves the readmitted-name ledger usable only
//! for discarding: callers abandon the whole rebuild.

use std::collections::{BTreeMap, BTreeSet};

use worth_foundational::facade::AggregateIdentity;

use crate::lineage::authority::{
    verify_persisted_birth_digest, BirthReadmissionRefusal, PersistedNameBytes, VerifiedBirthDigest,
};
use crate::lineage::data::{
    LineageEventKind, LineageEventRecord, PersistentNameIdentity, PersistentNameKey,
    PersistentNameSubjectKind,
};
use crate::lineage::index::PersistentNameCommittedBatch;

use super::frame_decoding::{DecodedCarriers, DecodedCommitFrame, DecodedLineageEvent};
use super::refusal::{PersistentNameDurabilityError as Refusal, PersistentNameReferenceRole};

/// Identities readmitted from verified persisted births, by their bytes.
#[derive(Default)]
pub(super) struct ReadmittedNames {
    births: BTreeMap<PersistedNameBytes, PersistentNameIdentity>,
}

impl ReadmittedNames {
    fn get(&self, raw: &PersistedNameBytes) -> Option<PersistentNameIdentity> {
        self.births.get(raw).copied()
    }

    fn require(
        &self,
        raw: &PersistedNameBytes,
        role: PersistentNameReferenceRole,
    ) -> Result<PersistentNameIdentity, Refusal> {
        self.get(raw)
            .ok_or(Refusal::UnverifiedBareIdentity { raw: *raw, role })
    }
}

/// Readmits one decoded frame as the canonical batch it persisted.
///
/// Complexity: O(frame births * birth size + references * log readmitted).
pub(super) fn readmit_commit_frame(
    frame: DecodedCommitFrame,
    readmitted: &mut ReadmittedNames,
) -> Result<PersistentNameCommittedBatch, Refusal> {
    let mut verified = Vec::with_capacity(frame.events.len());
    for mut event in frame.events {
        require_naming_shape(&event)?;
        let birth = match event.birth.take() {
            Some(claim) => {
                let digest = verify_persisted_birth_digest(claim).map_err(birth_refusal)?;
                readmitted
                    .births
                    .insert(*digest.identity().as_bytes(), digest.identity());
                Some(digest)
            }
            None => None,
        };
        verified.push((event, birth));
    }
    let mut lineage_events = Vec::with_capacity(verified.len());
    for (event, birth) in verified {
        lineage_events.push(bind_event(event, birth, readmitted)?);
    }
    let mut carriers = BTreeMap::new();
    for declaration in frame.carriers {
        let (key, names) = bind_carriers(declaration, readmitted)?;
        carriers.insert(key, names);
    }
    Ok(PersistentNameCommittedBatch {
        commit: frame.commit,
        lineage_events,
        entity_holders: frame.entity_holders,
        relation_holders: frame.relation_holders,
        group_events: frame.group_events.into_iter().collect(),
        carriers,
    })
}

/// Births belong only to `Create`; kept names only to the kinds that carry
/// one name across a record change.
fn require_naming_shape(event: &DecodedLineageEvent) -> Result<(), Refusal> {
    let refusal = |detail| Refusal::InvalidEventShape {
        event_id: event.event_id,
        detail,
    };
    if event.birth.is_some() && event.kind != LineageEventKind::Create {
        return Err(refusal("only a Create can carry a name birth"));
    }
    let keeps_name = matches!(
        event.kind,
        LineageEventKind::Create
            | LineageEventKind::Replace
            | LineageEventKind::Merge
            | LineageEventKind::Restore
    );
    if event.kept_name.is_some() && !keeps_name {
        return Err(refusal(
            "only Create, Replace, Merge, or Restore can keep a name",
        ));
    }
    Ok(())
}

fn bind_event(
    event: DecodedLineageEvent,
    birth: Option<VerifiedBirthDigest>,
    readmitted: &ReadmittedNames,
) -> Result<LineageEventRecord, Refusal> {
    let name_birth = match birth {
        Some(digest) => Some(
            digest
                .bind_references(|raw| readmitted.get(raw))
                .map_err(birth_refusal)?,
        ),
        None => None,
    };
    let kept_name = match &event.kept_name {
        Some(raw) => Some(readmitted.require(raw, PersistentNameReferenceRole::KeptName)?),
        None => None,
    };
    Ok(LineageEventRecord {
        event_id: event.event_id,
        commit: event.commit,
        branch_id: event.branch_id,
        kind: event.kind,
        sources: event.sources,
        targets: event.targets,
        subject: event.subject,
        name_birth,
        kept_name,
    })
}

fn bind_carriers(
    declaration: DecodedCarriers,
    readmitted: &ReadmittedNames,
) -> Result<(PersistentNameKey, BTreeSet<PersistentNameIdentity>), Refusal> {
    let key = match declaration.subject {
        PersistentNameSubjectKind::Entity => PersistentNameKey::Entity(
            readmitted.require(&declaration.key, PersistentNameReferenceRole::CarrierKey)?,
        ),
        PersistentNameSubjectKind::Relation => PersistentNameKey::Relation(
            readmitted.require(&declaration.key, PersistentNameReferenceRole::CarrierKey)?,
        ),
        PersistentNameSubjectKind::Group => {
            PersistentNameKey::Group(AggregateIdentity::from_bytes(declaration.key))
        }
    };
    let mut names = BTreeSet::new();
    for raw in &declaration.carriers {
        names.insert(readmitted.require(raw, PersistentNameReferenceRole::Carrier)?);
    }
    Ok((key, names))
}

fn birth_refusal(refusal: BirthReadmissionRefusal) -> Refusal {
    match refusal {
        BirthReadmissionRefusal::NonCanonicalParents { stored } => {
            Refusal::NonCanonicalBirthParents { identity: stored }
        }
        BirthReadmissionRefusal::MalformedDiscriminator { stored } => {
            Refusal::MalformedBirthDiscriminator { identity: stored }
        }
        BirthReadmissionRefusal::DigestMismatch { stored, recomputed } => {
            Refusal::CorruptBirthDigest { stored, recomputed }
        }
        BirthReadmissionRefusal::UnverifiedParent { parent, .. } => {
            Refusal::UnverifiedBareIdentity {
                raw: parent,
                role: PersistentNameReferenceRole::BirthParent,
            }
        }
        BirthReadmissionRefusal::UnverifiedDiscriminatorReference { reference, .. } => {
            Refusal::UnverifiedBareIdentity {
                raw: reference,
                role: PersistentNameReferenceRole::DiscriminatorReference,
            }
        }
    }
}
