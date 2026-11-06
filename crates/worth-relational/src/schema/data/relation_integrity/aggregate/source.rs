//! Strict physical legacy objects and explicit registered-derived source objects.
use crate::derived_relation::DerivedRelationIdentity;
use crate::identity::data::KindId;
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum AggregateIdentitySource {
    Orbit(KindId),
    Components {
        relations: Vec<KindId>,
    },
    DerivedOrbit {
        derived_orbit: DerivedRelationIdentity,
    },
    DerivedComponents {
        derived_components: DerivedRelationIdentity,
    },
}

impl Serialize for AggregateIdentitySource {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeMap;
        if let Self::Orbit(relation) = self {
            return relation.serialize(serializer);
        }
        // An object is mandatory even for compact struct serializers.
        let mut map = serializer.serialize_map(Some(1))?;
        match self {
            Self::Orbit(_) => unreachable!("scalar handled above"),
            Self::Components { relations } => map.serialize_entry("relations", relations)?,
            Self::DerivedOrbit { derived_orbit } => {
                map.serialize_entry("derived_orbit", derived_orbit)?
            }
            Self::DerivedComponents { derived_components } => {
                map.serialize_entry("derived_components", derived_components)?
            }
        }
        map.end()
    }
}

mod decoding;

impl AggregateIdentitySource {
    pub fn orbit_relation(&self) -> Option<KindId> {
        if let Self::Orbit(kind) = self {
            Some(*kind)
        } else {
            None
        }
    }

    pub fn stored_relations(&self) -> Option<&[KindId]> {
        match self {
            Self::Orbit(kind) => Some(std::slice::from_ref(kind)),
            Self::Components { relations } => Some(relations),
            Self::DerivedOrbit { .. } | Self::DerivedComponents { .. } => None,
        }
    }

    pub fn derived_relation(&self) -> Option<&DerivedRelationIdentity> {
        match self {
            Self::DerivedOrbit { derived_orbit } => Some(derived_orbit),
            Self::DerivedComponents { derived_components } => Some(derived_components),
            _ => None,
        }
    }

    pub(crate) fn contains_stored_relation(&self, kind: KindId) -> bool {
        self.stored_relations()
            .is_some_and(|relations| relations.binary_search(&kind).is_ok())
    }

    pub(crate) fn canonicalize(&mut self) {
        if let Self::Components { relations } = self {
            relations.sort();
            relations.dedup();
        }
    }

    pub(crate) fn owned_bytes(&self) -> u64 {
        match self {
            Self::Orbit(_) => 0,
            Self::Components { relations } => {
                (relations.capacity() * std::mem::size_of::<KindId>()) as u64
            }
            Self::DerivedOrbit { derived_orbit } => {
                (derived_orbit.schema.0.capacity() + derived_orbit.name.capacity()) as u64
            }
            Self::DerivedComponents { derived_components } => {
                (derived_components.schema.0.capacity() + derived_components.name.capacity()) as u64
            }
        }
    }

    pub(super) fn try_write_canonical(
        &self,
        emit: &mut dyn FnMut(&[u8]) -> Result<(), crate::execution::RelationalExecutionDenialCause>,
    ) -> Result<(), crate::execution::RelationalExecutionDenialCause> {
        match self {
            Self::Orbit(kind) => emit(&kind.0.to_be_bytes())?,
            Self::Components { relations } => {
                emit(b"WORTH_COMPONENT_SOURCES_V1\0")?;
                emit(&(relations.len() as u64).to_be_bytes())?;
                for kind in relations {
                    emit(&kind.0.to_be_bytes())?;
                }
            }
            Self::DerivedOrbit { derived_orbit } => {
                emit(b"WORTH_DERIVED_ORBIT_SOURCE_V1\0")?;
                crate::derived_relation::canonical::try_write_identity(derived_orbit, emit)?;
            }
            Self::DerivedComponents { derived_components } => {
                emit(b"WORTH_DERIVED_COMPONENT_SOURCE_V1\0")?;
                crate::derived_relation::canonical::try_write_identity(derived_components, emit)?;
            }
        }
        Ok(())
    }
}
