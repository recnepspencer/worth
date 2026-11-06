use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use worth_relational::facade::history::BranchId;
use worth_relational::facade::runtime::RelationalRuntime;

use crate::{WorthQueryEvidenceIdentity, WorthQueryEvidenceScope, WorthQueryEvidenceTag};

use super::{EffectExecutionOracleError, EffectExecutionOracleErrorKind};

fn relational_branch_observation_subject(
    branch: &BranchId,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    Ok(
        WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::EffectIntentReceipt)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "relational_branch_observation_subject_v1",
            )
            .field_shape(WorthQueryEvidenceTag::new("branch"), branch.0.as_str())
            .seal()?,
    )
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RelationalExecutionOracle {
    branch_identity: String,
    observed_commit_id: u64,
    observed_version_id: u64,
    observed_parent_commit_ids: Vec<u64>,
    relational_oracle_identity: WorthQueryEvidenceIdentity,
}

impl RelationalExecutionOracle {
    pub fn new(
        branch_identity: impl Into<String>,
        observed_commit_id: u64,
        observed_version_id: u64,
        observed_parent_commit_ids: Vec<u64>,
    ) -> Result<Self, IdentityDerivationDenial> {
        let branch_identity = branch_identity.into();
        let relational_oracle_identity = compose_relational_oracle_identity(
            &branch_identity,
            observed_commit_id,
            observed_version_id,
            &observed_parent_commit_ids,
        )?;
        Ok(Self {
            branch_identity,
            observed_commit_id,
            observed_version_id,
            observed_parent_commit_ids,
            relational_oracle_identity,
        })
    }

    pub fn observe_branch_head(
        runtime: &RelationalRuntime,
        branch: &BranchId,
    ) -> Result<Self, EffectExecutionOracleError> {
        let identity = runtime.branch_identity(branch).map_err(|_| {
            EffectExecutionOracleError::new(
                EffectExecutionOracleErrorKind::RelationalObservationMissingBranchHead,
                format!(
                    "independent relational oracle inspection could not resolve branch head for `{}`",
                    branch.0
                ),
                &relational_branch_observation_subject(branch),
                None,
            )
        })?;
        let (_, basis) = runtime.observe_branch(&identity).map_err(|_| {
            EffectExecutionOracleError::new(
                EffectExecutionOracleErrorKind::RelationalObservationMissingBranchHead,
                format!(
                    "independent relational oracle could not admit branch `{}`",
                    branch.0
                ),
                &relational_branch_observation_subject(branch),
                None,
            )
        })?;
        let observation = basis.observation();
        let history = runtime.history();
        let observed = history
            .branch_head_for_observation(&observation)
            .map_err(|_| {
                EffectExecutionOracleError::new(
                    EffectExecutionOracleErrorKind::RelationalObservationMissingBranchHead,
                    format!(
                        "independent relational oracle rejected branch basis `{}`",
                        branch.0
                    ),
                    &relational_branch_observation_subject(branch),
                    None,
                )
            })?
            .ok_or_else(|| {
                EffectExecutionOracleError::new(
                    EffectExecutionOracleErrorKind::RelationalObservationMissingBranchHead,
                    format!(
                        "independent relational oracle inspection found no head for `{}`",
                        branch.0
                    ),
                    &relational_branch_observation_subject(branch),
                    None,
                )
            })?;
        Ok(Self::new(
            branch.0.clone(),
            observed.commit_id.0,
            observed.version_id.0,
            observed.parents.iter().map(|parent| parent.0).collect(),
        ))
    }

    pub fn branch_identity(&self) -> &str {
        &self.branch_identity
    }

    pub fn observed_commit_id(&self) -> u64 {
        self.observed_commit_id
    }

    pub fn observed_version_id(&self) -> u64 {
        self.observed_version_id
    }

    pub fn observed_parent_commit_ids(&self) -> &[u64] {
        &self.observed_parent_commit_ids
    }

    pub fn relational_oracle_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.relational_oracle_identity
    }

    pub fn relational_oracle_for_reporting(&self) -> &str {
        self.relational_oracle_identity.as_str()
    }
}

fn compose_relational_oracle_identity(
    branch_identity: &str,
    observed_commit_id: u64,
    observed_version_id: u64,
    observed_parent_commit_ids: &[u64],
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    Ok(
        WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::EffectIntentReceipt)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "relational_execution_oracle_v1",
            )
            .field_shape(WorthQueryEvidenceTag::new("branch"), branch_identity)
            .field_usize(
                WorthQueryEvidenceTag::new("commit_id"),
                observed_commit_id as usize,
            )
            .field_usize(
                WorthQueryEvidenceTag::new("version_id"),
                observed_version_id as usize,
            )
            .field_value_sequence(
                WorthQueryEvidenceTag::new("parent_commit_ids"),
                observed_parent_commit_ids
                    .iter()
                    .map(|parent| parent.to_string()),
            )
            .seal()?,
    )
}
