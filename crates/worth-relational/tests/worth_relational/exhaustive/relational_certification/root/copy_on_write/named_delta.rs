#[allow(unused_imports)]
pub(crate) use crate::fast::relational_certification::root_named_delta_cow::*;

#[test]
fn every_supply_chain_delta_shares_payloads_within_its_declared_partition_footprint() {
    let host_request =
        crate::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let scenarios = [
        ("storm", BranchLabel::Storm, DeltaId::StormRerouteAurora),
        (
            "maintenance",
            BranchLabel::Maintenance,
            DeltaId::MaintainAtlasBerth,
        ),
        (
            "medical-hold",
            BranchLabel::MedicalHold,
            DeltaId::HoldMedicalCargo,
        ),
        (
            "southpoint-expansion",
            BranchLabel::SouthpointExpansion,
            DeltaId::ExpandSouthpointCapacity,
        ),
        (
            "competing-arrival",
            BranchLabel::CompetingArrival,
            DeltaId::CompetingAuroraArrival,
        ),
        (
            "inspection",
            BranchLabel::Inspection,
            DeltaId::RetireAtlasWhileInspectingAurora,
        ),
        ("rewire", BranchLabel::Rewire, DeltaId::RewireAuroraPortCall),
        (
            "hazard-v2",
            BranchLabel::HazardV2,
            DeltaId::AdoptHazardClassificationV2,
        ),
    ];
    for (branch, label, delta) in scenarios {
        prove_named_delta_cow(execution, branch, label, delta);
    }
}
