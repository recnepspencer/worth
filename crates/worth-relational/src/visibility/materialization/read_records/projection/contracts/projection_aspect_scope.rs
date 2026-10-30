use serde::{Deserialize, Deserializer, Serialize, Serializer};
use worth_foundational::facade::{
    AspectKey, AspectMask, AspectMaskLocator, CanonicalFieldPath, FieldKey, LocatorAuthority,
    ProjectionMask,
};

use super::projection_mask_basis::{
    prepare_projection_mask_for_canonical_basis, ProjectionMaskCanonicalBasis,
};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ProjectionAspectScope {
    aspects: Vec<ProjectionAspectRequirement>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectionAspectRequirement {
    aspect_key: AspectKey,
    mask: AspectMask<ProjectionMask>,
    locator: AspectMaskLocator<ProjectionMask>,
    mask_basis: Option<ProjectionMaskCanonicalBasis>,
}

impl ProjectionAspectScope {
    pub fn owned_allocation_capacity_bytes(&self) -> usize {
        self.aspects
            .capacity()
            .saturating_mul(std::mem::size_of::<ProjectionAspectRequirement>())
            .saturating_add(self.aspects.iter().fold(0usize, |total, requirement| {
                total.saturating_add(requirement.owned_allocation_capacity_bytes())
            }))
    }

    pub fn empty() -> Self {
        Self::default()
    }

    pub fn whole_aspects(aspects: impl IntoIterator<Item = AspectKey>) -> Self {
        Self::from_requirements(
            aspects
                .into_iter()
                .map(ProjectionAspectRequirement::whole_aspect),
        )
    }

    pub fn fields(aspect_key: AspectKey, fields: impl IntoIterator<Item = FieldKey>) -> Self {
        Self::from_requirements([ProjectionAspectRequirement::fields(aspect_key, fields)])
    }

    pub fn from_requirements(
        requirements: impl IntoIterator<Item = ProjectionAspectRequirement>,
    ) -> Self {
        let mut requirements = requirements.into_iter().collect::<Vec<_>>();
        requirements.sort_by(projection_requirement_order);
        requirements.dedup();
        Self {
            aspects: requirements,
        }
    }

    pub fn requirements(&self) -> &[ProjectionAspectRequirement] {
        &self.aspects
    }

    pub fn contains_whole_aspect(&self, aspect_key: &AspectKey) -> bool {
        self.aspects.iter().any(|requirement| {
            requirement.aspect_key == *aspect_key && requirement.mask.is_whole_aspect()
        })
    }

    pub fn contains_field(&self, aspect_key: &AspectKey, field: &FieldKey) -> bool {
        self.aspects.iter().any(|requirement| {
            requirement.aspect_key == *aspect_key
                && !requirement.mask.is_whole_aspect()
                && requirement
                    .mask
                    .paths()
                    .iter()
                    .any(|path| path == &CanonicalFieldPath::single(field.clone()))
        })
    }

    pub fn is_empty(&self) -> bool {
        self.aspects.is_empty()
    }
}

impl ProjectionAspectRequirement {
    fn owned_allocation_capacity_bytes(&self) -> usize {
        let basis_bytes = self.mask_basis.as_ref().map_or(0, |basis| {
            basis
                .payload()
                .owned_allocation_capacity_bytes()
                .saturating_add(
                    basis
                        .basis()
                        .basis()
                        .value()
                        .owned_allocation_capacity_bytes(),
                )
        });
        self.aspect_key
            .owned_allocation_capacity_bytes()
            .saturating_add(self.mask.owned_allocation_capacity_bytes())
            .saturating_add(self.locator.owned_allocation_capacity_bytes())
            .saturating_add(basis_bytes)
    }

    pub fn whole_aspect(aspect_key: AspectKey) -> Self {
        Self::new(aspect_key, AspectMask::<ProjectionMask>::whole_aspect())
    }

    pub fn fields(aspect_key: AspectKey, fields: impl IntoIterator<Item = FieldKey>) -> Self {
        Self::new(
            aspect_key,
            AspectMask::new(fields.into_iter().map(CanonicalFieldPath::single)),
        )
    }

    pub fn aspect_key(&self) -> &AspectKey {
        &self.aspect_key
    }

    pub fn mask(&self) -> &AspectMask<ProjectionMask> {
        &self.mask
    }

    pub fn locator(&self) -> &AspectMaskLocator<ProjectionMask> {
        &self.locator
    }

    pub fn mask_basis(&self) -> Option<&ProjectionMaskCanonicalBasis> {
        self.mask_basis.as_ref()
    }

    fn new(aspect_key: AspectKey, mask: AspectMask<ProjectionMask>) -> Self {
        let locator =
            AspectMaskLocator::projection(LocatorAuthority::Projected, aspect_key.clone(), &mask);
        let mask_basis = prepare_projection_mask_for_canonical_basis(&aspect_key, &mask);
        Self {
            aspect_key,
            mask,
            locator,
            mask_basis,
        }
    }
}

impl Serialize for ProjectionAspectScope {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        ProjectionAspectScopeWire::from(self).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for ProjectionAspectScope {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        ProjectionAspectScopeWire::deserialize(deserializer).map(Self::from)
    }
}

#[derive(Serialize, Deserialize)]
struct ProjectionAspectScopeWire {
    requirements: Vec<ProjectionAspectRequirementWire>,
}

#[derive(Serialize, Deserialize)]
struct ProjectionAspectRequirementWire {
    aspect_key: AspectKey,
    fields: Vec<FieldKey>,
}

impl From<&ProjectionAspectScope> for ProjectionAspectScopeWire {
    fn from(scope: &ProjectionAspectScope) -> Self {
        Self {
            requirements: scope
                .requirements()
                .iter()
                .map(ProjectionAspectRequirementWire::from)
                .collect(),
        }
    }
}

impl From<&ProjectionAspectRequirement> for ProjectionAspectRequirementWire {
    fn from(requirement: &ProjectionAspectRequirement) -> Self {
        Self {
            aspect_key: requirement.aspect_key().clone(),
            fields: requirement
                .mask()
                .paths()
                .iter()
                .flat_map(|path| path.fields().iter().cloned())
                .collect(),
        }
    }
}

impl From<ProjectionAspectScopeWire> for ProjectionAspectScope {
    fn from(wire: ProjectionAspectScopeWire) -> Self {
        ProjectionAspectScope::from_requirements(wire.requirements.into_iter().map(|requirement| {
            if requirement.fields.is_empty() {
                ProjectionAspectRequirement::whole_aspect(requirement.aspect_key)
            } else {
                ProjectionAspectRequirement::fields(requirement.aspect_key, requirement.fields)
            }
        }))
    }
}

fn projection_requirement_order(
    left: &ProjectionAspectRequirement,
    right: &ProjectionAspectRequirement,
) -> std::cmp::Ordering {
    left.aspect_key
        .cmp(&right.aspect_key)
        .then_with(|| mask_order_key(&left.mask).cmp(&mask_order_key(&right.mask)))
}

fn mask_order_key(mask: &AspectMask<ProjectionMask>) -> Vec<String> {
    if mask.is_whole_aspect() {
        return Vec::new();
    }
    mask.paths()
        .iter()
        .map(|path| {
            path.fields()
                .iter()
                .map(|field| field.as_str())
                .collect::<Vec<_>>()
                .join(".")
        })
        .collect()
}
