//! Encode path-keyed versions as pairs for formats requiring string map keys.
use std::collections::BTreeMap;

use serde::{de::Error, Deserialize, Deserializer, Serialize, Serializer};

use super::{PathVersions, ScopePath};

pub(super) fn serialize<S>(
    paths: &BTreeMap<ScopePath, PathVersions>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    paths.iter().collect::<Vec<_>>().serialize(serializer)
}

pub(super) fn deserialize<'de, D>(
    deserializer: D,
) -> Result<BTreeMap<ScopePath, PathVersions>, D::Error>
where
    D: Deserializer<'de>,
{
    let pairs = Vec::<(ScopePath, PathVersions)>::deserialize(deserializer)?;
    let mut paths = BTreeMap::new();
    for (path, versions) in pairs {
        if paths.insert(path, versions).is_some() {
            return Err(D::Error::custom("duplicate aspect-version scope path"));
        }
    }
    Ok(paths)
}
