//! The shared payment court explicitly authorizes its named P0 deployer.
pub use super::{principal_id, APPROVER};
pub use bank_server::{BankApplication, BankProgramDeploymentGrantSeed};
pub fn approver_grant() -> BankProgramDeploymentGrantSeed {
    BankProgramDeploymentGrantSeed::for_program::<BankApplication>(
        "approver-p0-deployment",
        principal_id(APPROVER),
        principal_id(APPROVER),
        0,
        u64::MAX,
    )
    .expect("the fixture explicitly grants only the named source-program actor")
}
