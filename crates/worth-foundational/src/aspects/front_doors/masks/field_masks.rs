use super::super::vocabulary::AspectFrontDoorConstructionDenial;
use crate::{
    AspectMask, CanonicalFieldPath, DiagnosticMask, FieldKey, MutationMask, ProjectionMask,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ProjectionMaskFrontDoor;

impl ProjectionMaskFrontDoor {
    pub fn whole_aspect(self) -> AspectMask<ProjectionMask> {
        AspectMask::whole_aspect()
    }

    pub fn fields(
        self,
        keys: impl IntoIterator<Item = impl Into<String>>,
    ) -> Result<AspectMask<ProjectionMask>, AspectFrontDoorConstructionDenial> {
        build_single_field_mask(keys)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct MutationMaskFrontDoor;

impl MutationMaskFrontDoor {
    pub fn whole_aspect(self) -> AspectMask<MutationMask> {
        AspectMask::whole_aspect()
    }

    pub fn fields(
        self,
        keys: impl IntoIterator<Item = impl Into<String>>,
    ) -> Result<AspectMask<MutationMask>, AspectFrontDoorConstructionDenial> {
        build_single_field_mask(keys)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DiagnosticMaskFrontDoor;

impl DiagnosticMaskFrontDoor {
    pub fn whole_aspect(self) -> AspectMask<DiagnosticMask> {
        AspectMask::whole_aspect()
    }

    pub fn fields(
        self,
        keys: impl IntoIterator<Item = impl Into<String>>,
    ) -> Result<AspectMask<DiagnosticMask>, AspectFrontDoorConstructionDenial> {
        build_single_field_mask(keys)
    }
}

fn build_single_field_mask<Mode>(
    keys: impl IntoIterator<Item = impl Into<String>>,
) -> Result<AspectMask<Mode>, AspectFrontDoorConstructionDenial> {
    let mut keys = keys.into_iter();
    collect_single_field_paths(&mut || keys.next().map(Into::into)).map(AspectMask::new)
}

/// One compiled field-path walk. `Mode` only wraps the finished paths.
#[inline(never)]
fn collect_single_field_paths(
    next_key: &mut dyn FnMut() -> Option<String>,
) -> Result<Vec<CanonicalFieldPath>, AspectFrontDoorConstructionDenial> {
    let mut paths = Vec::new();
    while let Some(raw_key) = next_key() {
        let field_key = FieldKey::new(raw_key.clone())
            .ok_or(AspectFrontDoorConstructionDenial::InvalidFieldKey(raw_key))?;
        paths.push(CanonicalFieldPath::single(field_key));
    }
    Ok(paths)
}
