use std::fmt;

use worth_ui_platform_pulse::observation_contract::{
    PlatformPulseLifecycleObservation, PlatformPulseLifecycleObservationEnvelope,
    PlatformPulseVisualComparison, PlatformPulseVisualIdentityTraceObservation,
    PlatformPulseVisualPointTrace, PlatformPulseVisualSnapshotCaptured,
    PlatformPulseVisualSnapshotRetired,
};
mod evidence;

#[derive(Clone, Debug)]
pub(crate) struct ExecutableVisualSnapshotEvidence {
    sequence: u64,
    snapshot: PlatformPulseVisualSnapshotCaptured,
}

#[derive(Clone, Debug)]
pub(crate) struct ExecutableVisualTraceEvidence {
    sequence: u64,
    trace: PlatformPulseVisualPointTrace,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ExecutableVisualRetirementEvidence {
    sequence: u64,
    retirement: PlatformPulseVisualSnapshotRetired,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ExecutableVisualComparisonEvidence {
    sequence: u64,
    comparison: PlatformPulseVisualComparison,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum ExecutableVisualIdentityFailure {
    VisualContract(super::visual_contract_manifest::PlatformPulseVisualContractFailure),
    WrongEvent(&'static str),
    WrongSequence {
        expected: u64,
        observed: u64,
    },
    SnapshotAffinity,
    ComparisonSnapshotIdentity {
        expected: [u64; 2],
        observed: [u64; 2],
    },
    ComparisonMeaning {
        identity_rebound: bool,
        retained_pixels_differ: Option<bool>,
    },
    ComparisonCost {
        structural_entries_examined: u64,
        retained_pixel_bytes_examined: u64,
    },
    SnapshotExtent,
    SnapshotPixelBudget,
    SnapshotIndexCardinality {
        expected_visible: u64,
        observed_visible: u64,
        expected_hit_test: u64,
        observed_hit_test: u64,
    },
    SnapshotCost,
    PointContract,
    TargetIdentity,
    BackgroundIdentity,
    AuthoredName,
    IncompleteTrace,
    OverlayAffinity,
    ClearAffinity,
    RetirementAffinity {
        expected: [u64; 3],
        observed: [u64; 3],
        explicitly_superseded: bool,
        released_registered_resource: bool,
    },
    RefreshDidNotAdvanceContent {
        rebind_frame: u64,
        refresh_frame: u64,
    },
    NativeCaptureExtent,
    NativeProcessIdentity,
    BorderNotVisible {
        matching: usize,
        sampled: usize,
    },
    BorderStillVisible {
        matching: usize,
        sampled: usize,
    },
    TargetPixelChanged,
    BackgroundPixelChanged,
}

pub(crate) fn adjudicate_visual_snapshot(
    envelope: PlatformPulseLifecycleObservationEnvelope,
    expected_frame: u64,
    expected_sequence: u64,
) -> Result<ExecutableVisualSnapshotEvidence, ExecutableVisualIdentityFailure> {
    adjudicate_snapshot_at_sequence(envelope, expected_frame, expected_sequence)
}

pub(crate) fn adjudicate_successor_visual_snapshot(
    envelope: PlatformPulseLifecycleObservationEnvelope,
    expected_frame: u64,
    expected_sequence: u64,
) -> Result<ExecutableVisualSnapshotEvidence, ExecutableVisualIdentityFailure> {
    adjudicate_snapshot_at_sequence(envelope, expected_frame, expected_sequence)
}

fn adjudicate_snapshot_at_sequence(
    envelope: PlatformPulseLifecycleObservationEnvelope,
    expected_frame: u64,
    expected_sequence: u64,
) -> Result<ExecutableVisualSnapshotEvidence, ExecutableVisualIdentityFailure> {
    let manifest = super::visual_contract_manifest::checked_in_adjudication_contract()
        .map_err(ExecutableVisualIdentityFailure::VisualContract)?;
    require_sequence(&envelope, expected_sequence)?;
    let PlatformPulseLifecycleObservation::VisualSnapshotCaptured(snapshot) = envelope.outcome()
    else {
        return Err(ExecutableVisualIdentityFailure::WrongEvent(
            "visual snapshot captured",
        ));
    };
    if snapshot.affinity().frame() != expected_frame
        || snapshot.affinity().snapshot() == 0
        || snapshot.affinity().relation()
            != worth_ui_platform_pulse::observation_contract::PlatformPulseVisualSnapshotRelationObservation::Current
    {
        return Err(ExecutableVisualIdentityFailure::SnapshotAffinity);
    }
    let physical_extent = expected_physical_extent(snapshot, &manifest)?;
    if snapshot.captured_client_extent() != [0, 0, physical_extent[0], physical_extent[1]]
        || snapshot.coordinates().client_physical_dimensions() != physical_extent
        || snapshot.pixels().dimensions() != physical_extent
    {
        return Err(ExecutableVisualIdentityFailure::SnapshotExtent);
    }
    let expected_stride = physical_extent[0]
        .checked_mul(4)
        .ok_or(ExecutableVisualIdentityFailure::SnapshotPixelBudget)?;
    let expected_bytes = u64::from(expected_stride)
        .checked_mul(u64::from(physical_extent[1]))
        .ok_or(ExecutableVisualIdentityFailure::SnapshotPixelBudget)?;
    if snapshot.pixels().stride() != expected_stride
        || snapshot.pixels().byte_count() != expected_bytes
        || expected_bytes > manifest.maximum_pixel_bytes()
    {
        return Err(ExecutableVisualIdentityFailure::SnapshotPixelBudget);
    }
    if snapshot.visible_region_count() != manifest.visible_region_count()
        || snapshot.hit_test_region_count() != manifest.hit_test_region_count()
    {
        return Err(ExecutableVisualIdentityFailure::SnapshotIndexCardinality {
            expected_visible: manifest.visible_region_count(),
            observed_visible: snapshot.visible_region_count(),
            expected_hit_test: manifest.hit_test_region_count(),
            observed_hit_test: snapshot.hit_test_region_count(),
        });
    }
    let cost = snapshot.cost_counters();
    if cost[4] != expected_bytes
        || cost[5] != expected_bytes
        || cost[6] != expected_bytes
        || cost[9] == 0
    {
        return Err(ExecutableVisualIdentityFailure::SnapshotCost);
    }
    Ok(ExecutableVisualSnapshotEvidence {
        sequence: expected_sequence,
        snapshot: snapshot.clone(),
    })
}

pub(crate) fn adjudicate_visual_trace(
    envelope: PlatformPulseLifecycleObservationEnvelope,
    snapshot: &ExecutableVisualSnapshotEvidence,
) -> Result<ExecutableVisualTraceEvidence, ExecutableVisualIdentityFailure> {
    let manifest = super::visual_contract_manifest::checked_in_adjudication_contract()
        .map_err(ExecutableVisualIdentityFailure::VisualContract)?;
    let sequence = snapshot.sequence.saturating_add(1);
    require_sequence(&envelope, sequence)?;
    let PlatformPulseLifecycleObservation::VisualPointTrace(trace) = envelope.outcome() else {
        return Err(ExecutableVisualIdentityFailure::WrongEvent(
            "visual point trace",
        ));
    };
    if trace.snapshot() != snapshot.snapshot.affinity().snapshot()
        || trace.target().point()
            != snapshot.project_logical_point(manifest.target_logical_point())?
        || trace.background().point()
            != snapshot.project_logical_point(manifest.background_logical_point())?
        || trace.target().visible_region() != snapshot.expected_target_region()?
    {
        return Err(ExecutableVisualIdentityFailure::PointContract);
    }
    require_same_resolution_identity(trace.target().visible(), trace.target().hit())
        .map_err(|_| ExecutableVisualIdentityFailure::TargetIdentity)?;
    require_same_resolution_identity(trace.background().visible(), trace.background().hit())
        .map_err(|_| ExecutableVisualIdentityFailure::BackgroundIdentity)?;
    if trace.target().hit().mounted().node_receipt()
        == trace.background().hit().mounted().node_receipt()
    {
        return Err(ExecutableVisualIdentityFailure::BackgroundIdentity);
    }
    if trace.target().hit().authored_semantic_name() != manifest.target_authored_name() {
        return Err(ExecutableVisualIdentityFailure::AuthoredName);
    }
    require_complete_trace(trace.target().visible())?;
    require_complete_trace(trace.target().hit())?;
    require_complete_trace(trace.background().visible())?;
    require_complete_trace(trace.background().hit())?;
    Ok(ExecutableVisualTraceEvidence {
        sequence,
        trace: trace.clone(),
    })
}

fn expected_physical_extent(
    snapshot: &PlatformPulseVisualSnapshotCaptured,
    manifest: &super::visual_contract_manifest::PlatformPulseVisualAdjudicationContract,
) -> Result<[u32; 2], ExecutableVisualIdentityFailure> {
    let logical = float_pair(snapshot.coordinates().viewport_logical_dimension_bits());
    let extent = manifest.logical_client_extent();
    if logical != [extent[0] as f32, extent[1] as f32] {
        return Err(ExecutableVisualIdentityFailure::SnapshotExtent);
    }
    let scale = float_pair(snapshot.coordinates().scale_bits());
    if scale.iter().any(|value| {
        !value.is_finite() || *value <= 0.0 || *value > manifest.maximum_capture_scale() as f32
    }) {
        return Err(ExecutableVisualIdentityFailure::SnapshotExtent);
    }
    Ok([
        project_axis(extent[0], scale[0], 0.0)?,
        project_axis(extent[1], scale[1], 0.0)?,
    ])
}

fn float_pair(bits: [u32; 2]) -> [f32; 2] {
    [f32::from_bits(bits[0]), f32::from_bits(bits[1])]
}

fn project_axis(
    logical: u32,
    scale: f32,
    translation: f32,
) -> Result<u32, ExecutableVisualIdentityFailure> {
    let physical = logical as f64 * f64::from(scale) + f64::from(translation);
    if !physical.is_finite() || physical < 0.0 || physical > f64::from(u32::MAX) {
        return Err(ExecutableVisualIdentityFailure::SnapshotExtent);
    }
    Ok(physical.round() as u32)
}

pub(crate) fn adjudicate_visual_retirement(
    envelope: PlatformPulseLifecycleObservationEnvelope,
    snapshot: &ExecutableVisualSnapshotEvidence,
    successor_frame: u64,
    expected_sequence: u64,
) -> Result<ExecutableVisualRetirementEvidence, ExecutableVisualIdentityFailure> {
    require_sequence(&envelope, expected_sequence)?;
    let PlatformPulseLifecycleObservation::VisualSnapshotRetired(retirement) = envelope.outcome()
    else {
        return Err(ExecutableVisualIdentityFailure::WrongEvent(
            "visual snapshot retired",
        ));
    };
    let expected = [
        snapshot.snapshot.affinity().snapshot(),
        snapshot.snapshot.affinity().frame(),
        successor_frame,
    ];
    let observed = [
        retirement.snapshot(),
        retirement.predecessor_frame(),
        retirement.successor_frame(),
    ];
    if observed != expected
        || !retirement.explicitly_superseded()
        || !retirement.released_registered_resource()
    {
        return Err(ExecutableVisualIdentityFailure::RetirementAffinity {
            expected,
            observed,
            explicitly_superseded: retirement.explicitly_superseded(),
            released_registered_resource: retirement.released_registered_resource(),
        });
    }
    Ok(ExecutableVisualRetirementEvidence {
        sequence: expected_sequence,
        retirement: *retirement,
    })
}

pub(crate) fn adjudicate_visual_comparison(
    envelope: PlatformPulseLifecycleObservationEnvelope,
    predecessor: &ExecutableVisualSnapshotEvidence,
    successor: &ExecutableVisualSnapshotEvidence,
    expected_sequence: u64,
) -> Result<ExecutableVisualComparisonEvidence, ExecutableVisualIdentityFailure> {
    require_sequence(&envelope, expected_sequence)?;
    let PlatformPulseLifecycleObservation::VisualComparison(comparison) = envelope.outcome() else {
        return Err(ExecutableVisualIdentityFailure::WrongEvent(
            "visual comparison",
        ));
    };
    let expected = [
        predecessor.snapshot.affinity().snapshot(),
        successor.snapshot.affinity().snapshot(),
    ];
    let observed = [
        comparison.predecessor_snapshot(),
        comparison.successor_snapshot(),
    ];
    if observed != expected || observed[0] == observed[1] {
        return Err(
            ExecutableVisualIdentityFailure::ComparisonSnapshotIdentity { expected, observed },
        );
    }
    if comparison.identity_rebound() || comparison.retained_pixels_differ() != Some(true) {
        return Err(ExecutableVisualIdentityFailure::ComparisonMeaning {
            identity_rebound: comparison.identity_rebound(),
            retained_pixels_differ: comparison.retained_pixels_differ(),
        });
    }
    if comparison.structural_entries_examined() == 0
        || comparison.structural_entries_examined() > 128
        || comparison.retained_pixel_bytes_examined() == 0
    {
        return Err(ExecutableVisualIdentityFailure::ComparisonCost {
            structural_entries_examined: comparison.structural_entries_examined(),
            retained_pixel_bytes_examined: comparison.retained_pixel_bytes_examined(),
        });
    }
    Ok(ExecutableVisualComparisonEvidence {
        sequence: expected_sequence,
        comparison: *comparison,
    })
}

fn require_sequence(
    envelope: &PlatformPulseLifecycleObservationEnvelope,
    expected: u64,
) -> Result<(), ExecutableVisualIdentityFailure> {
    let observed = envelope.sequence().value();
    if observed == expected {
        Ok(())
    } else {
        Err(ExecutableVisualIdentityFailure::WrongSequence { expected, observed })
    }
}

fn require_same_resolution_identity(
    visible: &PlatformPulseVisualIdentityTraceObservation,
    hit: &PlatformPulseVisualIdentityTraceObservation,
) -> Result<(), ()> {
    (visible.mounted() == hit.mounted() && visible.declaration() == hit.declaration())
        .then_some(())
        .ok_or(())
}

fn require_complete_trace(
    trace: &PlatformPulseVisualIdentityTraceObservation,
) -> Result<(), ExecutableVisualIdentityFailure> {
    let mounted = trace.mounted();
    if mounted.node_receipt() == 0
        || mounted.mounted_instance() == 0
        || mounted.incarnation() == 0
        || trace.graph_node() == 0
        || trace.declaration() == 0
        || trace.authored_semantic_name().is_empty()
        || trace.source_artifact_path().is_empty()
        || trace.evidence().is_empty()
    {
        Err(ExecutableVisualIdentityFailure::IncompleteTrace)
    } else {
        Ok(())
    }
}

impl fmt::Display for ExecutableVisualIdentityFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{self:?}")
    }
}
