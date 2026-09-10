use worth_foundational::{FoundationalBranchId, FoundationalBranchIdConstructionDenial};

pub type SignalBranchIdentity = FoundationalBranchId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SignalBranchIdentityConstructionDenial {
    InvalidBranchId(FoundationalBranchIdConstructionDenial),
    EmptyOwnerComponent,
}

/// Owner-validated branch name accepted by exact service forks.
///
/// Construction remains behind [`validate_signal_branch_name`]; descriptive
/// strings and branch identities cannot be promoted into this value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatedSignalBranchName(String);

impl ValidatedSignalBranchName {
    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub(crate) fn into_inner(self) -> String {
        self.0
    }
}

pub fn validate_signal_branch_name(
    branch_name: impl Into<String>,
) -> Result<ValidatedSignalBranchName, SignalBranchIdentityConstructionDenial> {
    let branch_name = branch_name.into();
    encode_owner_component(&branch_name)?;
    Ok(ValidatedSignalBranchName(branch_name))
}

pub fn signal_branch_identity(
    graph_instance_id: impl AsRef<str>,
    branch_id: u64,
    branch_name: impl AsRef<str>,
) -> Result<SignalBranchIdentity, SignalBranchIdentityConstructionDenial> {
    let graph_instance_id = encode_owner_component(graph_instance_id.as_ref())?;
    let branch_id = encode_owner_component(&branch_id.to_string())?;
    let branch_name = encode_owner_component(branch_name.as_ref())?;
    FoundationalBranchId::new(format!(
        "signal/{graph_instance_id}/{branch_id}/{branch_name}"
    ))
    .map_err(SignalBranchIdentityConstructionDenial::InvalidBranchId)
}

fn encode_owner_component(
    component: &str,
) -> Result<String, SignalBranchIdentityConstructionDenial> {
    if component.trim().is_empty() {
        return Err(SignalBranchIdentityConstructionDenial::EmptyOwnerComponent);
    }
    Ok(format!("{}:{component}", component.len()))
}
