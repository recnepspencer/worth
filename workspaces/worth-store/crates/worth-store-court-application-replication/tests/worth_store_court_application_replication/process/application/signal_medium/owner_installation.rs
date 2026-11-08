//! A refused peer installation releases the actual original Signal owner claim.
use super::{medium_fixture as media, owner_fixture as fixture};
use worth_execution::ExecutionRequest;
use worth_signal::facade::durability::*;

struct RefuseSecondBranch {
    visited: Vec<worth_foundational::facade::DurableSignalBranchIdentity>,
}
impl SignalAuthoredOwnerInstallation for RefuseSecondBranch {
    fn readmit_branch(
        &mut self,
        branch: SignalAuthoredBranchInstallation<'_>,
        _execution: &ExecutionRequest<'_, '_>,
    ) -> Result<(), SignalAuthoredInstallationRefused> {
        self.visited.push(branch.descriptor().branch());
        assert_eq!(branch.contracts().len(), 1);
        assert!(branch.bridge_section().is_empty());
        if self.visited.len() == 2 {
            Err(SignalAuthoredInstallationRefused)
        } else {
            Ok(())
        }
    }
}
#[test]
fn refused_second_branch_installation_releases_the_original_owner_for_complete_retry() {
    let directory = tempfile::tempdir().unwrap();
    let funding = media::serial_request();
    let execution = ExecutionRequest::serial(&funding);
    let (serving, medium) = media::open(directory.path(), true, &execution);
    fixture::seed(medium.as_ref(), &execution);
    let mut installation = RefuseSecondBranch {
        visited: Vec::new(),
    };
    let refused = fixture::program().0.readmit_authored_owner(
        medium.clone(),
        SignalRecordedIdentityContribution::empty(media::owner()),
        fixture::vocabulary(),
        &mut installation,
        &execution,
    );
    assert!(
        matches!(refused,Err(SignalOwnerReadmissionDenial::InstallationRefused(branch))
        if branch==installation.visited[1])
    );
    assert_eq!(installation.visited.len(), 2);
    assert!(installation.visited[0] < installation.visited[1]);
    let owner = fixture::readmit(medium.clone(), &execution);
    assert_eq!(owner.source_port().branches().unwrap().len(), 3);
    drop(owner);
    media::close(serving, medium, &execution);
}
