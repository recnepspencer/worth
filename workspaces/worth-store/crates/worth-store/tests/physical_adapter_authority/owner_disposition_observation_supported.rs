use worth_store::physical_runtime::{
    DamagedPhysicalDerivedDisposition, PhysicalArtifactDisposition,
    PhysicalArtifactRoleDisposition,
};

fn inspect_owner_observation(disposition: PhysicalArtifactDisposition) {
    let _ = disposition.validator_outcome();
    match disposition.owner_role() {
        Some(PhysicalArtifactRoleDisposition::IntactAuthority(observation)) => {
            let _ = observation.scope();
        }
        Some(PhysicalArtifactRoleDisposition::DamagedAuthority(observation)) => {
            let _ = observation.localization();
        }
        Some(PhysicalArtifactRoleDisposition::IntactDerived(observation)) => {
            let _ = observation.derived_scope();
            let _ = observation.authoritative_basis_scope();
        }
        Some(PhysicalArtifactRoleDisposition::DamagedDerived(disposition)) => match disposition {
            DamagedPhysicalDerivedDisposition::RebuildableDerived(observation) => {
                let _ = observation.damaged_derived_scope();
                let _ = observation.intact_authoritative_basis_scope();
            }
            DamagedPhysicalDerivedDisposition::Unknown(observation) => {
                let _ = observation.damaged_derived_scope();
            }
            DamagedPhysicalDerivedDisposition::Indeterminate(observation) => {
                let _ = observation.damaged_derived_scope();
            }
        }
        None => {}
    }
}

fn main() {
    let _ = inspect_owner_observation;
}
