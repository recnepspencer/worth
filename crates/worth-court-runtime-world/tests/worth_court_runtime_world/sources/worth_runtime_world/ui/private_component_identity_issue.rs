use worth_relational::facade::branch::RelationalBranchBasisAdmissionIdentity;
use worth_runtime_bridge::facade::BridgeCorrespondenceAdmissionIdentity;
use worth_signal::facade::branch::SignalBranchBasisAdmissionIdentity;

fn main() {
    let _ = RelationalBranchBasisAdmissionIdentity::issue();
    let _ = SignalBranchBasisAdmissionIdentity::issue();
    let _ = BridgeCorrespondenceAdmissionIdentity::issue();
}
