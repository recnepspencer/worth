use crate::access::shape::{AccessLaneClassification, AccessShape, AccessShapeDetail};
use crate::integrity::{layout_corruption, LayoutCorruptionOutcome};
use crate::strategy::{AdmittedLayoutStrategy, StrategyRebuildSourceRequirement};
use worth_store_wal::record_kind_admits_recovery_replay;

use super::super::denial::DerivedIndexRebuildDenied;
use super::super::outcome::RebuildAdmissionDenial;
use super::super::plan::DerivedIndexRebuildRequest;
use super::{DerivedIndexAuthoritySource, DerivedIndexRebuildSourceInput};

pub(in crate::maintenance::rebuild) fn validate_rebuild_shape(
    request: &DerivedIndexRebuildRequest,
) -> Result<(), RebuildAdmissionDenial> {
    if request.rebuild_shape().shape() != AccessShape::RebuildRead
        || !matches!(
            request.rebuild_shape().detail(),
            AccessShapeDetail::RebuildRead(_)
        )
        || request.rebuild_shape().lane() != AccessLaneClassification::Maintenance
    {
        return Err(RebuildAdmissionDenial::shape(
            DerivedIndexRebuildDenied::RebuildShapeRequired {
                family: request.strategy_family(),
            },
        ));
    }
    Ok(())
}

pub(in crate::maintenance::rebuild) fn admit_source_authority(
    request: &DerivedIndexRebuildRequest,
    admitted_strategy: AdmittedLayoutStrategy,
    shape_coverage: crate::materialization::LayoutCoverageWitness,
) -> Result<DerivedIndexAuthoritySource, RebuildAdmissionDenial> {
    let family = request.lifecycle().declaration().family();
    let requirement = admitted_strategy.rebuild_source_requirement();
    if let DerivedIndexRebuildSourceInput::WalReplayRecord { source_witness } =
        request.source_input()
        && !record_kind_admits_recovery_replay(source_witness.record().identity().kind())
    {
        return Err(RebuildAdmissionDenial::source_strategy(
            DerivedIndexRebuildDenied::SourceArtifactDoesNotMatchStrategy {
                required: requirement,
                source: "wal_replay_record_kind",
            },
        ));
    }
    let source_authority = DerivedIndexAuthoritySource::declare(
        requirement,
        family,
        shape_coverage.clone(),
        request.source_input(),
    )
    .ok_or_else(|| source_strategy_denial(requirement, request.source_input()))?;
    validate_source_binding(request)?;

    Ok(source_authority)
}

fn validate_source_binding(
    request: &DerivedIndexRebuildRequest,
) -> Result<(), RebuildAdmissionDenial> {
    let family = request.admitted_family();
    let materialization = request.materialization().source().kind();
    let source = match request.source_input() {
        DerivedIndexRebuildSourceInput::PhysicalRootManifest { source } => {
            if source.store_authority_identity() != family.authority_identity() {
                return Err(RebuildAdmissionDenial::source_authority(
                    DerivedIndexRebuildDenied::SourceStoreAuthorityMismatch {
                        expected: family.authority_identity(),
                        actual: source.store_authority_identity(),
                    },
                ));
            }
            crate::LayoutMaterializationSourceKind::BTreeRoot(source.witness().root_reference())
        }
        DerivedIndexRebuildSourceInput::WalReplayRecord { source_witness } => {
            let metadata = source_witness.security_metadata();
            if metadata.security_identity() != family.security_identity() {
                return Err(RebuildAdmissionDenial::source_security(
                    DerivedIndexRebuildDenied::SourceSecurityScopeMismatch {
                        expected: family.security_identity(),
                        actual: metadata.security_identity(),
                    },
                ));
            }
            if metadata.authority_identity() != family.authority_identity() {
                return Err(RebuildAdmissionDenial::source_authority(
                    DerivedIndexRebuildDenied::SourceStoreAuthorityMismatch {
                        expected: family.authority_identity(),
                        actual: metadata.authority_identity(),
                    },
                ));
            }
            crate::LayoutMaterializationSourceKind::LsmReplacement(
                source_witness.record().identity(),
            )
        }
        DerivedIndexRebuildSourceInput::DerivedProjectionRows
        | DerivedIndexRebuildSourceInput::CertificationRows
        | DerivedIndexRebuildSourceInput::DiagnosticReport
        | DerivedIndexRebuildSourceInput::JsonProjection
        | DerivedIndexRebuildSourceInput::TerminalProjection => return Ok(()),
    };
    if materialization != source {
        return Err(RebuildAdmissionDenial::source_identity(
            DerivedIndexRebuildDenied::SourceMaterializationIdentityMismatch {
                materialization,
                source,
            },
        ));
    }
    Ok(())
}

pub(in crate::maintenance::rebuild) fn classify_corruption(
    _source_authority: &DerivedIndexAuthoritySource,
) -> LayoutCorruptionOutcome {
    layout_corruption().assess_derived_projection(
        super::super::corruption::LayoutCorruptionClassification::derived_projection_rebuild_to_parity(),
    )
}

fn source_strategy_denial(
    requirement: StrategyRebuildSourceRequirement,
    source_input: &DerivedIndexRebuildSourceInput,
) -> RebuildAdmissionDenial {
    match source_input {
        DerivedIndexRebuildSourceInput::PhysicalRootManifest { .. } => {
            RebuildAdmissionDenial::source_strategy(
                DerivedIndexRebuildDenied::SourceArtifactDoesNotMatchStrategy {
                    required: requirement,
                    source: "physical_root_manifest",
                },
            )
        }
        DerivedIndexRebuildSourceInput::WalReplayRecord { .. } => {
            RebuildAdmissionDenial::source_strategy(
                DerivedIndexRebuildDenied::SourceArtifactDoesNotMatchStrategy {
                    required: requirement,
                    source: "wal_replay_record",
                },
            )
        }
        source => RebuildAdmissionDenial::source_not_authority(
            DerivedIndexRebuildDenied::SourceInputIsNotAuthority {
                source: Box::new(source.clone()),
            },
        ),
    }
}
