use super::{LayoutAdmissionDenial, LayoutAdmissionRequest, LayoutRequestedCapability};
use crate::keyspace::{CompositeKeyOrderingLaw, HashCollisionLaw};
use crate::maintenance::{IndexMaintenanceMode, PhysicalMutationShape};
use crate::strategy::{AdmittedLayoutStrategy, LayoutStrategyFamily};

pub(super) fn require_materialization_compatibility(
    admitted: AdmittedLayoutStrategy,
    request: &LayoutAdmissionRequest,
) -> Result<(), LayoutAdmissionDenial> {
    let strategy_family = admitted.admitted_family().declaration().family();
    if !request.requires_exact_materialization() {
        return Ok(());
    }

    let coverage = request
        .exact_coverage()
        .ok_or(LayoutAdmissionDenial::ExactMaterializationRequired)?;
    if coverage.family() != strategy_family {
        return Err(LayoutAdmissionDenial::CoverageFamilyDoesNotMatchStrategy {
            coverage_family: coverage.family(),
            strategy_family,
        });
    }
    coverage
        .require_exact()
        .map_err(LayoutAdmissionDenial::ExactCoverageDenied)?;
    Ok(())
}

pub(crate) const fn family_lane_supports_operation(
    family_lane: crate::catalog::ArtifactFamilyAccessLane,
    operation_lane: crate::catalog::ArtifactFamilyAccessLane,
) -> bool {
    use crate::catalog::ArtifactFamilyAccessLane;

    match family_lane {
        ArtifactFamilyAccessLane::HotPath => matches!(
            operation_lane,
            ArtifactFamilyAccessLane::HotPath
                | ArtifactFamilyAccessLane::MaintenancePath
                | ArtifactFamilyAccessLane::TerminalPath
        ),
        ArtifactFamilyAccessLane::MaintenancePath => {
            matches!(operation_lane, ArtifactFamilyAccessLane::MaintenancePath)
        }
        ArtifactFamilyAccessLane::VerifierPath => {
            matches!(operation_lane, ArtifactFamilyAccessLane::VerifierPath)
        }
        ArtifactFamilyAccessLane::TerminalPath => {
            matches!(operation_lane, ArtifactFamilyAccessLane::TerminalPath)
        }
    }
}

pub(super) fn require_key_law_compatibility(
    admitted: AdmittedLayoutStrategy,
    request: &LayoutAdmissionRequest,
) -> Result<(), LayoutAdmissionDenial> {
    let capability = request.requested_capability();
    if requires_comparator_law(capability) && admitted.comparator_law().is_none() {
        return Err(LayoutAdmissionDenial::ComparatorLawRequired {
            family: admitted.family(),
            capability,
        });
    }

    match capability {
        LayoutRequestedCapability::PointLookup if admitted.supports_point_access() => Ok(()),
        LayoutRequestedCapability::OrderedRange
            if admitted.supports_range_access()
                || admits_owned_maintenance_traversal(admitted, request) =>
        {
            if admitted.range_bound_law().is_none() {
                return Err(LayoutAdmissionDenial::RangeBoundLawRequired {
                    family: admitted.family(),
                });
            }
            Ok(())
        }
        LayoutRequestedCapability::PrefixTraversal if admitted.supports_prefix_access() => {
            if admitted.prefix_law().is_none() {
                return Err(LayoutAdmissionDenial::PrefixLawRequired {
                    family: admitted.family(),
                });
            }
            Ok(())
        }
        LayoutRequestedCapability::ExactScan
            if admitted.supports_scan_access()
                || admits_owned_maintenance_traversal(admitted, request) =>
        {
            Ok(())
        }
        LayoutRequestedCapability::BlobStreaming if admitted.supports_streaming_access() => Ok(()),
        _ => Err(
            LayoutAdmissionDenial::StrategyDoesNotSupportRequestedCapability {
                family: admitted.family(),
                capability,
            },
        ),
    }
}

const fn admits_owned_maintenance_traversal(
    admitted: AdmittedLayoutStrategy,
    request: &LayoutAdmissionRequest,
) -> bool {
    use crate::catalog::ArtifactFamilyAccessLane;

    matches!(
        request.requested_lane(),
        ArtifactFamilyAccessLane::MaintenancePath
    ) && match admitted.family() {
        LayoutStrategyFamily::BTreeRange => matches!(
            request.maintenance_mode(),
            IndexMaintenanceMode::RebuildOnly
        ),
        LayoutStrategyFamily::BaselineLsmWriteOptimized => {
            matches!(
                request.mutation_shape(),
                PhysicalMutationShape::CompactionRewrite
            ) || matches!(
                request.maintenance_mode(),
                IndexMaintenanceMode::RebuildOnly
            )
        }
        _ => false,
    }
}

const fn requires_comparator_law(capability: LayoutRequestedCapability) -> bool {
    !matches!(
        capability,
        LayoutRequestedCapability::ExactScan | LayoutRequestedCapability::BlobStreaming
    )
}

pub(super) fn require_requested_key_law_compatibility(
    admitted: AdmittedLayoutStrategy,
    request: &LayoutAdmissionRequest,
) -> Result<(Option<HashCollisionLaw>, Option<CompositeKeyOrderingLaw>), LayoutAdmissionDenial> {
    let hash_equality_law = request.required_key_laws().hash_equality_law();
    if let Some(law) = hash_equality_law {
        if law.domain() != admitted.key_domain() {
            return Err(
                LayoutAdmissionDenial::HashEqualityLawDoesNotMatchKeyDomain {
                    requested_domain: law.domain(),
                    strategy_domain: admitted.key_domain(),
                },
            );
        }
    }

    let composite_ordering_law = request.required_key_laws().composite_ordering_law();
    if let Some(law) = composite_ordering_law {
        if law.domain() != admitted.key_domain() {
            return Err(
                LayoutAdmissionDenial::CompositeOrderingLawDoesNotMatchKeyDomain {
                    requested_domain: law.domain(),
                    strategy_domain: admitted.key_domain(),
                },
            );
        }
    }

    Ok((hash_equality_law, composite_ordering_law))
}

pub(super) const fn maintenance_mode_supports_lane(
    mode: IndexMaintenanceMode,
    lane: crate::catalog::ArtifactFamilyAccessLane,
) -> bool {
    mode.supports_lane(lane)
}

pub(super) const fn mutation_shape_is_compatible(
    family: LayoutStrategyFamily,
    mutation_shape: PhysicalMutationShape,
) -> bool {
    matches!(
        (family, mutation_shape),
        (_, PhysicalMutationShape::ObservationOnly)
            | (
                LayoutStrategyFamily::BTreeRange,
                PhysicalMutationShape::PointRewrite
            )
            | (
                LayoutStrategyFamily::BaselineLsmWriteOptimized,
                PhysicalMutationShape::LogStructuredAppend
                    | PhysicalMutationShape::CompactionRewrite,
            )
    )
}
