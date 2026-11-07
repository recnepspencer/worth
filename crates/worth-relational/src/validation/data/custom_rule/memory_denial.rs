//! Durable carriage of Execution-owned memory refusal values.
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use worth_execution::{MemoryLimitDenial, MemoryLimitLevel};
#[derive(Serialize, Deserialize)]
enum Level {
    Policy { ancestor: u32 },
    Process,
    Declared,
}
#[derive(Serialize, Deserialize)]
enum Denial {
    Capacity {
        requested: u64,
        admitted: u64,
        level: Level,
    },
    CounterOverflow {
        requested: u64,
        charged: u64,
        level: Level,
    },
}
impl From<MemoryLimitLevel> for Level {
    fn from(value: MemoryLimitLevel) -> Self {
        match value {
            MemoryLimitLevel::Policy { ancestor } => Self::Policy { ancestor },
            MemoryLimitLevel::Process => Self::Process,
            MemoryLimitLevel::Declared => Self::Declared,
        }
    }
}
impl From<Level> for MemoryLimitLevel {
    fn from(value: Level) -> Self {
        match value {
            Level::Policy { ancestor } => Self::Policy { ancestor },
            Level::Process => Self::Process,
            Level::Declared => Self::Declared,
        }
    }
}
pub(super) fn serialize<S: Serializer>(
    value: &MemoryLimitDenial,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    let value = match *value {
        MemoryLimitDenial::Capacity {
            requested,
            admitted,
            level,
        } => Denial::Capacity {
            requested,
            admitted,
            level: level.into(),
        },
        MemoryLimitDenial::CounterOverflow {
            requested,
            charged,
            level,
        } => Denial::CounterOverflow {
            requested,
            charged,
            level: level.into(),
        },
    };
    value.serialize(serializer)
}
pub(super) fn deserialize<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<MemoryLimitDenial, D::Error> {
    Ok(match Denial::deserialize(deserializer)? {
        Denial::Capacity {
            requested,
            admitted,
            level,
        } => MemoryLimitDenial::Capacity {
            requested,
            admitted,
            level: level.into(),
        },
        Denial::CounterOverflow {
            requested,
            charged,
            level,
        } => MemoryLimitDenial::CounterOverflow {
            requested,
            charged,
            level: level.into(),
        },
    })
}
