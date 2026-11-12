//! Explicit tags for authored execution declarations.
use super::{Denial, Read, Write};
use crate::data::node::ContextRequirement;
pub(super) fn write_context(out: &mut Write<'_>, value: ContextRequirement) -> Result<(), Denial> {
    out.byte(match value {
        ContextRequirement::None => 0,
        ContextRequirement::DomainContext => 1,
        ContextRequirement::RelationalSnapshot => 2,
    })
}
pub(super) fn read_context(read: &mut Read<'_>) -> Result<ContextRequirement, Denial> {
    Ok(match read.byte()? {
        0 => ContextRequirement::None,
        1 => ContextRequirement::DomainContext,
        2 => ContextRequirement::RelationalSnapshot,
        _ => return Err(Denial::Malformed),
    })
}
use crate::data::performance::IdentityBasis;
pub(super) fn write_identity_basis(
    out: &mut Write<'_>,
    value: IdentityBasis,
) -> Result<(), Denial> {
    out.byte(match value {
        IdentityBasis::AspectVersion => 0,
        IdentityBasis::OutputIdentity => 1,
        IdentityBasis::ContinuityToken => 2,
        IdentityBasis::OutputIdentityAndContinuity => 3,
    })
}
pub(super) fn read_identity_basis(read: &mut Read<'_>) -> Result<IdentityBasis, Denial> {
    Ok(match read.byte()? {
        0 => IdentityBasis::AspectVersion,
        1 => IdentityBasis::OutputIdentity,
        2 => IdentityBasis::ContinuityToken,
        3 => IdentityBasis::OutputIdentityAndContinuity,
        _ => return Err(Denial::Malformed),
    })
}
use crate::data::performance::SuppressionBasis;
pub(super) fn write_suppression_basis(
    out: &mut Write<'_>,
    value: SuppressionBasis,
) -> Result<(), Denial> {
    out.byte(match value {
        SuppressionBasis::Never => 0,
        SuppressionBasis::OutputIdentity => 1,
        SuppressionBasis::ContinuityToken => 2,
        SuppressionBasis::ComparatorMatch => 3,
        SuppressionBasis::OutputIdentityAndComparator => 4,
    })
}
pub(super) fn read_suppression_basis(read: &mut Read<'_>) -> Result<SuppressionBasis, Denial> {
    Ok(match read.byte()? {
        0 => SuppressionBasis::Never,
        1 => SuppressionBasis::OutputIdentity,
        2 => SuppressionBasis::ContinuityToken,
        3 => SuppressionBasis::ComparatorMatch,
        4 => SuppressionBasis::OutputIdentityAndComparator,
        _ => return Err(Denial::Malformed),
    })
}
use crate::data::performance::CanonicalDependencyOrder;
pub(super) fn write_dependency_order(
    out: &mut Write<'_>,
    value: CanonicalDependencyOrder,
) -> Result<(), Denial> {
    out.byte(match value {
        CanonicalDependencyOrder::SourceAspectScope => 0,
    })
}
pub(super) fn read_dependency_order(
    read: &mut Read<'_>,
) -> Result<CanonicalDependencyOrder, Denial> {
    Ok(match read.byte()? {
        0 => CanonicalDependencyOrder::SourceAspectScope,
        _ => return Err(Denial::Malformed),
    })
}
use crate::data::performance::ComparatorBasis;
pub(super) fn write_comparator_basis(
    out: &mut Write<'_>,
    value: ComparatorBasis,
) -> Result<(), Denial> {
    out.byte(match value {
        ComparatorBasis::TierDefault => 0,
        ComparatorBasis::Exact => 1,
        ComparatorBasis::Tolerance => 2,
        ComparatorBasis::OutputIdentity => 3,
        ComparatorBasis::Custom => 4,
    })
}
pub(super) fn read_comparator_basis(read: &mut Read<'_>) -> Result<ComparatorBasis, Denial> {
    Ok(match read.byte()? {
        0 => ComparatorBasis::TierDefault,
        1 => ComparatorBasis::Exact,
        2 => ComparatorBasis::Tolerance,
        3 => ComparatorBasis::OutputIdentity,
        4 => ComparatorBasis::Custom,
        _ => return Err(Denial::Malformed),
    })
}
use crate::data::performance::PathClass;
pub(super) fn write_path_class(out: &mut Write<'_>, value: PathClass) -> Result<(), Denial> {
    out.byte(match value {
        PathClass::Operational => 0,
        PathClass::Rich => 1,
    })
}
pub(super) fn read_path_class(read: &mut Read<'_>) -> Result<PathClass, Denial> {
    Ok(match read.byte()? {
        0 => PathClass::Operational,
        1 => PathClass::Rich,
        _ => return Err(Denial::Malformed),
    })
}
use crate::data::performance::MaintenanceMode;
pub(super) fn write_maintenance(out: &mut Write<'_>, value: MaintenanceMode) -> Result<(), Denial> {
    out.byte(match value {
        MaintenanceMode::IncrementalOnly => 0,
        MaintenanceMode::RebuildAllowed => 1,
        MaintenanceMode::DensityAdaptive => 2,
    })
}
pub(super) fn read_maintenance(read: &mut Read<'_>) -> Result<MaintenanceMode, Denial> {
    Ok(match read.byte()? {
        0 => MaintenanceMode::IncrementalOnly,
        1 => MaintenanceMode::RebuildAllowed,
        2 => MaintenanceMode::DensityAdaptive,
        _ => return Err(Denial::Malformed),
    })
}
use crate::data::performance::ArtifactPolicyClass;
pub(super) fn write_artifact_policy(
    out: &mut Write<'_>,
    value: ArtifactPolicyClass,
) -> Result<(), Denial> {
    out.byte(match value {
        ArtifactPolicyClass::OperationalMinimal => 0,
        ArtifactPolicyClass::DevelopmentRetained => 1,
        ArtifactPolicyClass::ForensicReconstructable => 2,
    })
}
pub(super) fn read_artifact_policy(read: &mut Read<'_>) -> Result<ArtifactPolicyClass, Denial> {
    Ok(match read.byte()? {
        0 => ArtifactPolicyClass::OperationalMinimal,
        1 => ArtifactPolicyClass::DevelopmentRetained,
        2 => ArtifactPolicyClass::ForensicReconstructable,
        _ => return Err(Denial::Malformed),
    })
}
use crate::data::performance::AuthorityPolicy;
pub(super) fn write_authority_policy(
    out: &mut Write<'_>,
    value: AuthorityPolicy,
) -> Result<(), Denial> {
    out.byte(match value {
        AuthorityPolicy::AuthoritativeOnly => 0,
        AuthorityPolicy::SpeculativeThenReconcile => 1,
    })
}
pub(super) fn read_authority_policy(read: &mut Read<'_>) -> Result<AuthorityPolicy, Denial> {
    Ok(match read.byte()? {
        0 => AuthorityPolicy::AuthoritativeOnly,
        1 => AuthorityPolicy::SpeculativeThenReconcile,
        _ => return Err(Denial::Malformed),
    })
}
