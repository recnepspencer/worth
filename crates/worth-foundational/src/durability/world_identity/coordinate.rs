use super::DurableWorldOwnerIdentity;

/// Descriptive World coordinates keep explicit memory-only observations
/// separate from Home identity. Neither variant grants runtime authority.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum WorldOwnerCoordinate {
    Home { owner: DurableWorldOwnerIdentity },
    MemoryOnly { observation: u64 },
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct HomeCoordinateWire {
    version: crate::boundary_protocol::BoundaryProtocolVersion,
    owner: DurableWorldOwnerIdentity,
}

#[derive(serde::Deserialize)]
#[serde(untagged)]
enum CoordinateWire {
    MemoryOnly(u64),
    Home(HomeCoordinateWire),
}

impl serde::Serialize for WorldOwnerCoordinate {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match *self {
            Self::MemoryOnly { observation } => serializer.serialize_u64(observation),
            Self::Home { owner } => serde::Serialize::serialize(
                &HomeCoordinateWire {
                    version: crate::boundary_protocol::BoundaryProtocolVersion::new(1),
                    owner,
                },
                serializer,
            ),
        }
    }
}

impl<'de> serde::Deserialize<'de> for WorldOwnerCoordinate {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        match <CoordinateWire as serde::Deserialize>::deserialize(deserializer)? {
            CoordinateWire::MemoryOnly(observation) => Ok(Self::MemoryOnly { observation }),
            CoordinateWire::Home(wire) => {
                if wire.version.get() != 1 {
                    return Err(serde::de::Error::custom(
                        "unsupported World owner coordinate version",
                    ));
                }
                Ok(Self::Home { owner: wire.owner })
            }
        }
    }
}

impl WorldOwnerCoordinate {
    pub const fn home_owner(self) -> Option<DurableWorldOwnerIdentity> {
        match self {
            Self::Home { owner } => Some(owner),
            Self::MemoryOnly { .. } => None,
        }
    }

    /// The enclosing Home supplies its namespace. Memory-only rows retain
    /// their original process observation and never qualify durability.
    pub const fn row_owner_number(self) -> u64 {
        match self {
            Self::Home { owner } => owner.slot().get(),
            Self::MemoryOnly { observation } => observation,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::durability::{HomeIdentity, WorldOwnerSlotIdentity};

    #[test]
    fn memory_coordinates_keep_their_explicit_non_durable_number_encoding() {
        let value = WorldOwnerCoordinate::MemoryOnly { observation: 19 };
        assert_eq!(serde_json::to_string(&value).unwrap(), "19");
        assert_eq!(
            serde_json::from_str::<WorldOwnerCoordinate>("19").unwrap(),
            value
        );
    }

    #[test]
    fn home_coordinates_preserve_namespace_and_refuse_zero_or_unknown_versions() {
        let owner = DurableWorldOwnerIdentity::from_recorded(
            HomeIdentity::from_observed_bytes([1; 16]).unwrap(),
            WorldOwnerSlotIdentity::from_recorded(7),
        );
        let value = WorldOwnerCoordinate::Home { owner };
        let mut encoded = serde_json::to_value(value).unwrap();
        assert_eq!(
            serde_json::from_value::<WorldOwnerCoordinate>(encoded.clone()).unwrap(),
            value
        );
        encoded["version"] = serde_json::to_value(2).unwrap();
        assert!(serde_json::from_value::<WorldOwnerCoordinate>(encoded.clone()).is_err());
        encoded["version"] = serde_json::to_value(1).unwrap();
        encoded["owner"]["home"] = serde_json::to_value([0_u8; 16]).unwrap();
        assert!(serde_json::from_value::<WorldOwnerCoordinate>(encoded).is_err());
    }
}
