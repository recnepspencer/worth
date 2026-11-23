//! Verified serde adapters for persisted persistent-name fields on
//! [`super::events::LineageEventRecord`].
//!
//! A [`PersistentNameBirthRecord`] is NEVER deserialized into a trusted
//! [`PersistentNameIdentity`] without first passing
//! [`crate::lineage::authority::verify_persisted_birth_digest`], which re-hashes
//! every typed birth part and validates strictly sorted parents and canonical
//! discriminator wire framing.

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::lineage::authority::{verify_persisted_birth_digest, PersistedBirthClaim};
use crate::lineage::data::{
    PersistentNameBirthParts, PersistentNameBirthRecord, PersistentNameIdentity,
    PersistentNameSubjectKind,
};

#[derive(Serialize, Deserialize)]
struct PersistedBirthWire {
    identity: [u8; PersistentNameIdentity::BYTE_LEN],
    subject_kind: u8,
    scheme_id: String,
    occurrence_key: String,
    role_path: String,
    parent_identities: Vec<[u8; PersistentNameIdentity::BYTE_LEN]>,
    canonical_discriminator: Vec<u8>,
}

fn subject_to_tag(kind: PersistentNameSubjectKind) -> u8 {
    match kind {
        PersistentNameSubjectKind::Entity => 1,
        PersistentNameSubjectKind::Relation => 2,
        PersistentNameSubjectKind::Group => 3,
    }
}

fn subject_from_tag(tag: u8) -> Result<PersistentNameSubjectKind, String> {
    match tag {
        1 => Ok(PersistentNameSubjectKind::Entity),
        2 => Ok(PersistentNameSubjectKind::Relation),
        3 => Ok(PersistentNameSubjectKind::Group),
        other => Err(format!("invalid persistent name subject tag {other}")),
    }
}

pub(super) mod verified_birth {
    use super::*;

    pub(crate) fn serialize<S: Serializer>(
        value: &Option<PersistentNameBirthRecord>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        let wire = value.as_ref().map(|record| PersistedBirthWire {
            identity: *record.identity().as_bytes(),
            subject_kind: subject_to_tag(record.subject_kind()),
            scheme_id: record.parts().scheme_id().to_owned(),
            occurrence_key: record.parts().occurrence_key().to_owned(),
            role_path: record.parts().role_path().to_owned(),
            parent_identities: record
                .parts()
                .parent_identities()
                .iter()
                .map(|id| *id.as_bytes())
                .collect(),
            canonical_discriminator: record.parts().canonical_discriminator().to_vec(),
        });
        wire.serialize(serializer)
    }

    pub(crate) fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Option<PersistentNameBirthRecord>, D::Error> {
        let Some(wire) = Option::<PersistedBirthWire>::deserialize(deserializer)? else {
            return Ok(None);
        };
        let subject_kind = subject_from_tag(wire.subject_kind).map_err(serde::de::Error::custom)?;
        let parent_bytes = wire.parent_identities.clone();
        let claim = PersistedBirthClaim {
            stored_identity: wire.identity,
            subject_kind,
            scheme_id: wire.scheme_id.clone(),
            occurrence_key: wire.occurrence_key.clone(),
            role_path: wire.role_path.clone(),
            parent_identities: wire.parent_identities,
            canonical_discriminator: wire.canonical_discriminator.clone(),
        };
        let verified = verify_persisted_birth_digest(claim)
            .map_err(|refusal| serde::de::Error::custom(format!("corrupt persisted birth: {refusal:?}")))?;
        let parents = parent_bytes
            .into_iter()
            .map(PersistentNameIdentity::from_minted_digest)
            .collect();
        let parts = PersistentNameBirthParts::new(
            wire.scheme_id,
            wire.occurrence_key,
            wire.role_path,
            parents,
            wire.canonical_discriminator,
        );
        Ok(Some(PersistentNameBirthRecord::new(
            verified.identity(),
            subject_kind,
            parts,
        )))
    }
}

pub(super) mod persisted_kept_name {
    use super::*;

    pub(crate) fn serialize<S: Serializer>(
        value: &Option<PersistentNameIdentity>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        value.map(|id| *id.as_bytes()).serialize(serializer)
    }

    pub(crate) fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Option<PersistentNameIdentity>, D::Error> {
        let raw = Option::<[u8; PersistentNameIdentity::BYTE_LEN]>::deserialize(deserializer)?;
        Ok(raw.map(PersistentNameIdentity::from_minted_digest))
    }
}
