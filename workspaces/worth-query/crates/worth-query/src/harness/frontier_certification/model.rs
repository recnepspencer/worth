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

    pub fn has_required_outputs(&self) -> bool {
        !self.parity_bundle.query_digest().as_str().is_empty()
            && !self.parity_bundle.plan_digest().as_str().is_empty()
            && !self.parity_bundle.result_digest().as_str().is_empty()
            && !self.parity_bundle.basis_digest().is_empty()
            && !self
                .parity_bundle
                .route_posture_digest()
                .as_str()
                .is_empty()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FrontierCertificationRejection {
    pub failure_class: FrontierFailureClass,
    pub failure_digest: String,
}

impl FrontierCertificationRejection {
    pub fn has_required_outputs(&self) -> bool {
        !self.failure_digest.is_empty()
    }
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FrontierCloseoutStatus {
    Satisfied,
}

impl FrontierCloseoutStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Satisfied => "satisfied",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FrontierCloseoutRequirement {
    pub requirement_name: &'static str,
    pub status: FrontierCloseoutStatus,
    pub proof_artifacts: &'static [&'static str],
    pub certification_rows: &'static [&'static str],
    pub notes: &'static str,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MilestoneFivePointThreeFrontierCloseoutArtifact {
    pub suite_name: &'static str,
    pub closeout_matrix_digest: String,
    pub certification_bundle_digest: String,
    pub must_ship: Vec<FrontierCloseoutRequirement>,
    pub must_preserve: Vec<FrontierCloseoutRequirement>,
    pub proof_obligations: Vec<FrontierCloseoutRequirement>,
    pub acceptance_evidence: Vec<FrontierCloseoutRequirement>,
}

impl MilestoneFivePointThreeFrontierCloseoutArtifact {
    pub fn all_requirements_marked_satisfied(&self) -> bool {
        self.must_ship
            .iter()
            .chain(self.must_preserve.iter())
            .chain(self.proof_obligations.iter())
            .chain(self.acceptance_evidence.iter())
            .all(|requirement| requirement.status == FrontierCloseoutStatus::Satisfied)
    }
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

pub fn closeout_matrix_digest_parts(
    sections: &[(&str, &[FrontierCloseoutRequirement])],
    certification_bundle_digest: &str,
) -> Vec<String> {
    let mut parts = vec![format!(
        "certification_bundle_digest:{certification_bundle_digest}"
    )];
    for (section_name, requirements) in sections {
        parts.push(format!("section:{section_name}"));
        for requirement in *requirements {
            parts.push(format!("requirement:{}", requirement.requirement_name));
            parts.push(format!("status:{}", requirement.status.as_str()));
            for artifact in requirement.proof_artifacts {
                parts.push(format!("artifact:{artifact}"));
            }
            for row in requirement.certification_rows {
                parts.push(format!("row:{row}"));
            }
            parts.push(format!("notes:{}", requirement.notes));
        }
    }
    parts
}
