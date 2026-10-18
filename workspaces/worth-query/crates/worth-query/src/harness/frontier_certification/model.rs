use super::super::certification::{digest_parts, CertificationMatrix};
use crate::planning::{FrontierCounterSnapshot, FrontierParityBundle, PlannedRouteFamily};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum FrontierPerturbationClass {
    SerialControlParity,
    SerialFallbackParity,
    ExactBasisBundleParity,
    UnsupportedFrontierFamilyRejection,
    UnsupportedBundleCompositionRejection,
    MixedBasisBundleRejection,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FrontierRouteClass {
    SerialControl,
    SerialFallback,
    SerialFallbackBundle,
}

impl FrontierRouteClass {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::SerialControl => "serial_control",
            Self::SerialFallback => "serial_fallback",
            Self::SerialFallbackBundle => "serial_fallback_bundle",
        }
    }
}

impl From<&PlannedRouteFamily> for FrontierRouteClass {
    fn from(value: &PlannedRouteFamily) -> Self {
        match value {
            PlannedRouteFamily::FrontierSerialControl => Self::SerialControl,
            PlannedRouteFamily::FrontierSerialFallback => Self::SerialFallback,
            PlannedRouteFamily::FrontierSerialFallbackBundle => Self::SerialFallbackBundle,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FrontierFailureClass {
    UnsupportedFrontierFamily,
    UnsupportedBundleComposition,
    MixedBasisBundleDenied,
}

impl FrontierFailureClass {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::UnsupportedFrontierFamily => "unsupported-frontier-family",
            Self::UnsupportedBundleComposition => "unsupported-bundle-composition",
            Self::MixedBasisBundleDenied => "mixed-basis-bundle-denied",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FrontierCertificationLane {
    pub parity_bundle: FrontierParityBundle,
}

impl FrontierCertificationLane {
    pub fn route_class(&self) -> FrontierRouteClass {
        FrontierRouteClass::from(self.parity_bundle.route_family())
    }

    pub fn counter_snapshot(&self) -> &FrontierCounterSnapshot {
        self.parity_bundle.counter_snapshot()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FrontierCertificationRejection {
    pub failure_class: FrontierFailureClass,
    pub failure_digest: String,
}

pub type FrontierCertificationMatrix = CertificationMatrix<
    FrontierPerturbationClass,
    FrontierCertificationLane,
    FrontierCertificationRejection,
>;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MilestoneFivePointThreeFrontierCertificationArtifact {
    pub suite_name: &'static str,
    pub certification_bundle_digest: String,
    pub coverage_matrix_digest: String,
    pub matrix: FrontierCertificationMatrix,
}

impl FrontierCertificationMatrix {
    pub fn into_milestone_five_point_three_artifact(
        self,
    ) -> MilestoneFivePointThreeFrontierCertificationArtifact {
        let certification_bundle_digest = digest_parts(&bundle_digest_parts(&self));
        let coverage_matrix_digest = digest_parts(&coverage_digest_parts(&self));

        MilestoneFivePointThreeFrontierCertificationArtifact {
            suite_name: self.suite_name,
            certification_bundle_digest,
            coverage_matrix_digest,
            matrix: self,
        }
    }
}

fn bundle_digest_parts(matrix: &FrontierCertificationMatrix) -> Vec<String> {
    let mut parts = vec![format!("suite:{}", matrix.suite_name)];
    for row in &matrix.rows {
        parts.push(format!("canonical:{}", row.row_name));
        parts.extend(lane_digest_parts(&row.control_lane, "control"));
        parts.extend(lane_digest_parts(&row.hostile_lane, "hostile"));
        parts.extend(lane_digest_parts(&row.parity_lane, "parity"));
    }
    for row in &matrix.rejection_rows {
        parts.push(format!("rejection:{}", row.row_name));
        parts.extend(lane_digest_parts(&row.control_lane, "control"));
        parts.extend(rejection_digest_parts(
            &row.hostile_lane,
            "hostile_rejection",
        ));
        parts.extend(lane_digest_parts(&row.parity_lane, "parity"));
    }
    parts
}

fn coverage_digest_parts(matrix: &FrontierCertificationMatrix) -> Vec<String> {
    let mut parts = vec![format!("suite:{}", matrix.suite_name)];
    parts.extend(
        matrix
            .rows
            .iter()
            .map(|row| format!("canonical:{}", row.row_name)),
    );
    parts.extend(
        matrix
            .rejection_rows
            .iter()
            .map(|row| format!("rejection:{}", row.row_name)),
    );
    parts
}

fn lane_digest_parts(bundle: &FrontierCertificationLane, label: &str) -> Vec<String> {
    let mut parts = vec![
        format!(
            "{label}.query_digest:{}",
            bundle.parity_bundle.query_digest().as_str()
        ),
        format!(
            "{label}.plan_digest:{}",
            bundle.parity_bundle.plan_digest().as_str()
        ),
        format!(
            "{label}.result_digest:{}",
            bundle.parity_bundle.result_digest().as_str()
        ),
        format!(
            "{label}.basis_digest:{}",
            bundle.parity_bundle.basis_digest()
        ),
        format!("{label}.route_class:{}", bundle.route_class().as_str()),
        format!(
            "{label}.route_posture_digest:{}",
            bundle.parity_bundle.route_posture_digest().as_str()
        ),
        format!(
            "{label}.predicted_breadth:{}",
            bundle.parity_bundle.predicted_breadth().value()
        ),
        format!(
            "{label}.reported_execution_records_examined_count:{}",
            bundle
                .parity_bundle
                .reported_execution_records_examined_count()
        ),
    ];
    parts.extend(bundle.parity_bundle.counter_snapshot().digest_parts(label));
    parts
}

fn rejection_digest_parts(bundle: &FrontierCertificationRejection, label: &str) -> Vec<String> {
    vec![
        format!("{label}.failure_class:{}", bundle.failure_class.as_str()),
        format!("{label}.failure_digest:{}", bundle.failure_digest),
    ]
}
