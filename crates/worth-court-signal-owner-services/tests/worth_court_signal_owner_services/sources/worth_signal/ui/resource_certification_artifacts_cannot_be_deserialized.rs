use worth_signal::facade::{
    ResourceCertificationBundle, ResourceCertificationRecord, ResourceCertificationSummary,
    ResourceMilestoneBCertificationRun, ResourceMilestoneBHostileScenarioEvidence,
    ResourceMilestoneBHostileScenarioEvidenceRow, ResourceMilestoneBPerformanceCloseout,
    ResourceMilestoneBPerformanceCloseoutRow, ResourceMilestoneBPerformanceCloseoutSummary,
    ResourceMilestoneBScenarioMatrix, ResourceMilestoneBScenarioRow,
    ResourceMilestoneCCertificationRun, ResourceMilestoneCPolicyPerformanceCloseout,
    ResourceMilestoneCPolicyPerformanceCloseoutRow, ResourceMilestoneCPolicyScenarioMatrix,
    ResourceMilestoneCPolicyScenarioRow,
};

fn requires_deserialize_owned<T: serde::de::DeserializeOwned>() {}

fn main() {
    requires_deserialize_owned::<ResourceCertificationRecord>();
    requires_deserialize_owned::<ResourceCertificationSummary>();
    requires_deserialize_owned::<ResourceCertificationBundle>();
    requires_deserialize_owned::<ResourceMilestoneBHostileScenarioEvidenceRow>();
    requires_deserialize_owned::<ResourceMilestoneBHostileScenarioEvidence>();
    requires_deserialize_owned::<ResourceMilestoneBPerformanceCloseoutRow>();
    requires_deserialize_owned::<ResourceMilestoneBPerformanceCloseoutSummary>();
    requires_deserialize_owned::<ResourceMilestoneBPerformanceCloseout>();
    requires_deserialize_owned::<ResourceMilestoneBScenarioRow>();
    requires_deserialize_owned::<ResourceMilestoneBScenarioMatrix>();
    requires_deserialize_owned::<ResourceMilestoneBCertificationRun>();
    requires_deserialize_owned::<ResourceMilestoneCPolicyScenarioRow>();
    requires_deserialize_owned::<ResourceMilestoneCPolicyScenarioMatrix>();
    requires_deserialize_owned::<ResourceMilestoneCPolicyPerformanceCloseoutRow>();
    requires_deserialize_owned::<ResourceMilestoneCPolicyPerformanceCloseout>();
    requires_deserialize_owned::<ResourceMilestoneCCertificationRun>();
}
