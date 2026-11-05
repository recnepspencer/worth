use worth_query_admission::facade::basis::admit_basis_capability;
use worth_query_admission::facade::basis::evaluate_basis_observation_eligibility;
use worth_query_admission::facade::basis::normalize_raw_basis_intent;
use worth_query_admission::facade::basis::AdmittedBasisCapability;
use worth_query_admission::facade::basis::BasisOperationLane;
use worth_query_admission::facade::basis::ObservationLaneWitness;
use worth_query_admission::facade::basis::RawBasisIntent;

pub fn admitted_basis() -> AdmittedBasisCapability<ObservationLaneWitness> {
    admit(RawBasisIntent::CurrentHead)
}

pub fn admitted_alternate_basis() -> AdmittedBasisCapability<ObservationLaneWitness> {
    admit(RawBasisIntent::BranchHead {
        branch_identity: "alternate-convergence-basis".into(),
        accessible: true,
    })
}

pub fn admit(raw: RawBasisIntent) -> AdmittedBasisCapability<ObservationLaneWitness> {
    let normalized = normalize_raw_basis_intent(raw, ObservationLaneWitness::lane_name())
        .expect("fixture basis must normalize");
    admit_basis_capability(
        evaluate_basis_observation_eligibility(normalized)
            .expect("fixture basis must be observation eligible"),
    )
}
