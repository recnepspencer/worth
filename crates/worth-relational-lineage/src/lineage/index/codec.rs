//! Naming-page codec of the name index's rows.
use crate::contracts::lineage::data::name_resolution::PersistentNameTarget;
use crate::contracts::lineage::data::naming::{PersistentNameBirthParts, PersistentNameIdentity};
use crate::contracts::storage::data::storage_unavailable::RelationalStorageUnavailableCause as Cause;
use crate::lineage::naming_pages::codec::{read_serde, write_serde, NamingCodec, Reader, Writer};

use super::{HolderEpochs, LineageHolder, LiveName, NameEntry, NameOrigin, NameState};

impl NamingCodec for HolderEpochs {
    fn write(&self, out: &mut Writer) {
        self.0.write(out);
    }
    fn read(input: &mut Reader<'_>) -> Result<Self, Cause> {
        let map = im::OrdMap::<
            (
                crate::contracts::history::data::coordinates::CommitId,
                crate::contracts::identity::data::primitives::LineageId,
            ),
            Option<crate::contracts::history::data::coordinates::CommitId>,
        >::read(input)?;
        if map
            .iter()
            .any(|((bound_at, _), ended_at)| ended_at.is_some_and(|end| end < *bound_at))
        {
            return Err(Cause::Damaged);
        }
        Ok(Self(map))
    }
}

impl NamingCodec for LineageHolder {
    fn write(&self, out: &mut Writer) {
        match self {
            Self::Entity(entity) => (out.u8(0), write_serde(entity, out)).1,
            Self::Relation(relation) => (out.u8(1), write_serde(relation, out)).1,
        }
    }
    fn read(input: &mut Reader<'_>) -> Result<Self, Cause> {
        match input.u8()? {
            0 => Ok(Self::Entity(read_serde(input)?)),
            1 => Ok(Self::Relation(read_serde(input)?)),
            _ => Err(Cause::Damaged),
        }
    }
}

impl NamingCodec for PersistentNameTarget {
    fn write(&self, out: &mut Writer) {
        match self {
            Self::Entity {
                name,
                entity,
                lineage,
            } => {
                out.u8(0);
                name.write(out);
                write_serde(entity, out);
                lineage.write(out);
            }
            Self::Relation {
                name,
                relation,
                lineage,
            } => {
                out.u8(1);
                name.write(out);
                write_serde(relation, out);
                lineage.write(out);
            }
            Self::Group { group } => (out.u8(2), group.write(out)).1,
        }
    }
    fn read(input: &mut Reader<'_>) -> Result<Self, Cause> {
        match input.u8()? {
            0 => Ok(Self::Entity {
                name: NamingCodec::read(input)?,
                entity: read_serde(input)?,
                lineage: NamingCodec::read(input)?,
            }),
            1 => Ok(Self::Relation {
                name: NamingCodec::read(input)?,
                relation: read_serde(input)?,
                lineage: NamingCodec::read(input)?,
            }),
            2 => Ok(Self::Group {
                group: NamingCodec::read(input)?,
            }),
            _ => Err(Cause::Damaged),
        }
    }
}

impl NamingCodec for LiveName {
    fn write(&self, out: &mut Writer) {
        self.target.write(out);
        self.carriers.write(out);
    }
    fn read(input: &mut Reader<'_>) -> Result<Self, Cause> {
        Ok(Self {
            target: NamingCodec::read(input)?,
            carriers: NamingCodec::read(input)?,
        })
    }
}

impl NamingCodec for NameState {
    fn write(&self, out: &mut Writer) {
        match self {
            Self::Live(live) => (out.u8(0), live.write(out)).1,
            Self::Ended => out.u8(1),
        }
    }
    fn read(input: &mut Reader<'_>) -> Result<Self, Cause> {
        match input.u8()? {
            0 => Ok(Self::Live(NamingCodec::read(input)?)),
            1 => Ok(Self::Ended),
            _ => Err(Cause::Damaged),
        }
    }
}

impl NamingCodec for PersistentNameBirthParts {
    fn write(&self, out: &mut Writer) {
        self.scheme_id().to_owned().write(out);
        self.occurrence_key().to_owned().write(out);
        self.role_path().to_owned().write(out);
        self.parent_identities().to_vec().write(out);
        out.bytes(self.canonical_discriminator());
        out.u8(u8::from(self.structural_identity()));
    }
    fn read(input: &mut Reader<'_>) -> Result<Self, Cause> {
        let scheme_id = String::read(input)?;
        let occurrence_key = String::read(input)?;
        let role_path = String::read(input)?;
        let parents = Vec::<PersistentNameIdentity>::read(input)?;
        let discriminator = input.bytes()?.to_vec();
        let structural_identity = match input.u8()? {
            0 => false,
            1 => true,
            _ => return Err(Cause::Damaged),
        };
        Ok(<Self as crate::contracts::lineage::data::naming::PersistentNameBirthPartsOwnerPort>::new(
            scheme_id,
            occurrence_key,
            role_path,
            parents,
            discriminator,
            structural_identity,
        ))
    }
}

impl NamingCodec for NameOrigin {
    fn write(&self, out: &mut Writer) {
        match self {
            Self::Birth(parts) => (out.u8(0), parts.write(out)).1,
            Self::Group { scheme_id } => (out.u8(1), scheme_id.write(out)).1,
        }
    }
    fn read(input: &mut Reader<'_>) -> Result<Self, Cause> {
        match input.u8()? {
            0 => Ok(Self::Birth(NamingCodec::read(input)?)),
            1 => Ok(Self::Group {
                scheme_id: NamingCodec::read(input)?,
            }),
            _ => Err(Cause::Damaged),
        }
    }
}

impl NamingCodec for NameEntry {
    fn write(&self, out: &mut Writer) {
        self.origin.write(out);
        self.born.write(out);
        self.timeline.write(out);
        self.forward_edges.write(out);
        self.lineage_commits.write(out);
        if matches!(self.origin, NameOrigin::Group { .. }) {
            self.group_events.write(out);
        }
    }
    fn read(input: &mut Reader<'_>) -> Result<Self, Cause> {
        let origin = NameOrigin::read(input)?;
        let group = matches!(origin, NameOrigin::Group { .. });
        let entry = Self {
            origin,
            born: NamingCodec::read(input)?,
            timeline: NamingCodec::read(input)?,
            forward_edges: NamingCodec::read(input)?,
            lineage_commits: NamingCodec::read(input)?,
            group_events: if group {
                NamingCodec::read(input)?
            } else {
                Default::default()
            },
        };
        if entry
            .timeline
            .get_min()
            .is_none_or(|(first, _)| *first < entry.born)
        {
            return Err(Cause::Damaged);
        }
        Ok(entry)
    }
}
