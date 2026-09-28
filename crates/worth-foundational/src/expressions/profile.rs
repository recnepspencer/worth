//! Installed resource profiles. Profiles narrow; they never widen.

use super::denial::{
    ExpressionDenial, ExpressionDenialDetail, ExpressionResource, ExpressionResult,
};

const RESOURCE_COUNT: usize = 16;

const RESOURCES: [ExpressionResource; RESOURCE_COUNT] = [
    ExpressionResource::SourceBytes,
    ExpressionResource::SyntaxNodes,
    ExpressionResource::SyntaxDepth,
    ExpressionResource::CallDepth,
    ExpressionResource::AdmissionWork,
    ExpressionResource::ExpandedInstructions,
    ExpressionResource::DecodedBytes,
    ExpressionResource::CanonicalBytes,
    ExpressionResource::SemanticWork,
    ExpressionResource::VisitedElements,
    ExpressionResource::InputBytes,
    ExpressionResource::OutputBytes,
    ExpressionResource::ScratchBytes,
    ExpressionResource::RetainedConsumptionBytes,
    ExpressionResource::BitWidth,
    ExpressionResource::CatalogEntries,
];

const KIB: u64 = 1024;
const MIB: u64 = 1024 * KIB;

/// Structural ceilings shared by every profile: they bound native recursion.
const SYNTAX_DEPTH: u64 = 64;
const CALL_DEPTH: u64 = 32;
const BIT_WIDTH: u64 = 4096;

/// The admission and evaluation ceilings an owner installs.
///
/// Every phase checks its resources against the installed profile before it
/// allocates or recurses. [`ExpressionProfile::narrowed`] can lower a ceiling;
/// no operation raises one past the profile it started from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ExpressionProfile {
    limits: [u64; RESOURCE_COUNT],
}

impl ExpressionProfile {
    /// Ceilings for editor and interactive use.
    pub const fn interactive() -> Self {
        Self::from_structure(
            64 * KIB,
            8_192,
            [16_384, 1_000_000, 8 * MIB, MIB, 8 * MIB, 2 * MIB],
        )
    }

    /// Ceilings for batch engineering evaluation.
    pub const fn engineering() -> Self {
        Self::from_structure(
            MIB,
            65_536,
            [
                1_048_576,
                100_000_000,
                256 * MIB,
                64 * MIB,
                64 * MIB,
                64 * MIB,
            ],
        )
    }

    /// Derives admission ceilings from source and node ceilings, then takes the
    /// evaluation ceilings `[visited, work, input, output, scratch, retained]`.
    const fn from_structure(source: u64, nodes: u64, evaluation: [u64; 6]) -> Self {
        let [visited, work, input, output, scratch, retained] = evaluation;
        Self {
            limits: [
                source,
                nodes,
                SYNTAX_DEPTH,
                CALL_DEPTH,
                source + nodes * 256,
                nodes * 8,
                source * 4 + nodes * 32,
                source * 4 + nodes * 128,
                work,
                visited,
                input,
                output,
                scratch,
                retained,
                BIT_WIDTH,
                nodes / 2,
            ],
        }
    }

    /// The ceiling for `resource`.
    pub const fn limit(&self, resource: ExpressionResource) -> u64 {
        self.limits[resource_index(resource)]
    }

    /// A copy with `resource` lowered to `limit`.
    ///
    /// Denies a request that would raise the ceiling or set it to zero.
    pub fn narrowed(
        mut self,
        resource: ExpressionResource,
        limit: u64,
    ) -> Result<Self, ExpressionDenial> {
        let current = self.limit(resource);
        if limit == 0 || limit > current {
            return Err(ExpressionDenial::new(ExpressionDenialDetail::Bounds(
                "profile limits may only narrow to a nonzero ceiling",
            )));
        }
        self.limits[resource_index(resource)] = limit;
        Ok(self)
    }

    /// The tighter of `self` and `other` for every resource.
    pub(crate) fn meet(&self, other: &Self) -> Self {
        let mut limits = self.limits;
        for (limit, other) in limits.iter_mut().zip(other.limits) {
            *limit = (*limit).min(other);
        }
        Self { limits }
    }

    /// Every resource with its ceiling, in declaration order.
    pub fn limits(&self) -> impl Iterator<Item = (ExpressionResource, u64)> + '_ {
        RESOURCES.iter().copied().zip(self.limits.iter().copied())
    }
}

const fn resource_index(resource: ExpressionResource) -> usize {
    resource as usize
}

/// A monotone counter against one ceiling.
#[derive(Debug, Clone)]
pub(crate) struct AdmissionMeter {
    resource: ExpressionResource,
    limit: u64,
    used: u64,
}

impl AdmissionMeter {
    pub(crate) fn new(profile: &ExpressionProfile, resource: ExpressionResource) -> Self {
        Self {
            resource,
            limit: profile.limit(resource),
            used: 0,
        }
    }

    pub(crate) fn charge(&mut self, units: u64) -> ExpressionResult<()> {
        self.used = self.used.saturating_add(units);
        if self.used > self.limit {
            Err(ExpressionDenial::resource(self.resource, self.limit))
        } else {
            Ok(())
        }
    }

    pub(crate) fn used(&self) -> u64 {
        self.used
    }
}

/// Denies `actual` above the profile ceiling for `resource`.
pub(crate) fn check_limit(
    profile: &ExpressionProfile,
    resource: ExpressionResource,
    actual: u64,
) -> ExpressionResult<()> {
    let limit = profile.limit(resource);
    if actual > limit {
        Err(ExpressionDenial::resource(resource, limit))
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{ExpressionProfile, RESOURCES};
    use crate::expressions::denial::ExpressionResource;

    #[test]
    fn resource_table_matches_declaration_order() {
        for (index, resource) in RESOURCES.iter().enumerate() {
            assert_eq!(*resource as usize, index);
        }
    }

    #[test]
    fn profiles_narrow_but_never_widen() {
        let profile = ExpressionProfile::interactive();
        assert_eq!(profile.limit(ExpressionResource::SyntaxNodes), 8_192);
        assert_eq!(profile.limit(ExpressionResource::SourceBytes), 64 * 1024);
        let narrowed = profile
            .narrowed(ExpressionResource::SyntaxNodes, 16)
            .unwrap();
        assert_eq!(narrowed.limit(ExpressionResource::SyntaxNodes), 16);
        assert!(narrowed
            .narrowed(ExpressionResource::SyntaxNodes, 17)
            .is_err());
        assert!(profile.narrowed(ExpressionResource::BitWidth, 0).is_err());
        let engineering = ExpressionProfile::engineering();
        assert_eq!(
            engineering.limit(ExpressionResource::SemanticWork),
            100_000_000
        );
    }
}
