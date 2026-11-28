use serde::{Deserialize, Deserializer, Serialize, Serializer};
use worth_foundational::facade::{
    AspectKey, AspectMask, AspectMaskLocator, CanonicalFieldPath, DiagnosticMask, FieldKey,
    LocatorAuthority,
};

pub mod canonical_basis;

pub mod canonical_field_path {
    use super::*;
    use serde::de::Error;

    pub fn serialize<S>(path: &CanonicalFieldPath, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        path.fields().serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<CanonicalFieldPath, D::Error>
    where
        D: Deserializer<'de>,
    {
        field_path_from_native(Vec::<FieldKey>::deserialize(deserializer)?)
            .map_err(D::Error::custom)
    }
}

pub mod diagnostic_mask {
    use super::*;

    pub fn serialize<S>(mask: &AspectMask<DiagnosticMask>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        field_paths_to_native(mask.paths()).serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<AspectMask<DiagnosticMask>, D::Error>
    where
        D: Deserializer<'de>,
    {
        field_paths_from_native(Vec::<Vec<FieldKey>>::deserialize(deserializer)?)
            .map(AspectMask::new)
            .map_err(serde::de::Error::custom)
    }
}

pub mod diagnostic_mask_locator {
    use super::*;

    #[derive(Serialize, Deserialize)]
    struct NativeDiagnosticMaskLocator {
        authority: LocatorAuthority,
        aspect_key: AspectKey,
        paths: Vec<Vec<FieldKey>>,
    }

    pub fn serialize<S>(
        locator: &AspectMaskLocator<DiagnosticMask>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        NativeDiagnosticMaskLocator {
            authority: locator.authority(),
            aspect_key: locator.aspect_key().clone(),
            paths: field_paths_to_native(locator.paths()),
        }
        .serialize(serializer)
    }

    pub fn deserialize<'de, D>(
        deserializer: D,
    ) -> Result<AspectMaskLocator<DiagnosticMask>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let locator = NativeDiagnosticMaskLocator::deserialize(deserializer)?;
        let mask = AspectMask::new(
            field_paths_from_native(locator.paths).map_err(serde::de::Error::custom)?,
        );
        Ok(AspectMaskLocator::diagnostic(
            locator.authority,
            locator.aspect_key,
            &mask,
        ))
    }
}

pub(super) fn field_paths_to_native(paths: &[CanonicalFieldPath]) -> Vec<Vec<FieldKey>> {
    paths.iter().map(field_path_to_native).collect()
}

pub(super) fn field_path_to_native(path: &CanonicalFieldPath) -> Vec<FieldKey> {
    path.fields().to_vec()
}

pub(super) fn field_paths_from_native(
    paths: impl IntoIterator<Item = Vec<FieldKey>>,
) -> Result<Vec<CanonicalFieldPath>, String> {
    paths.into_iter().map(field_path_from_native).collect()
}

pub(super) fn field_path_from_native(fields: Vec<FieldKey>) -> Result<CanonicalFieldPath, String> {
    CanonicalFieldPath::new(fields)
        .ok_or_else(|| "diagnostic field path cannot be empty".to_string())
}
