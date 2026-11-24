use crate::publication::cdc::data::SubscriberCheckpoint;
use serde::{Deserialize, Serialize};

use worth_relational_contracts::internal_port::publication::cdc::data::subscriber_contract::SubscriberContractDeclaration;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubscriberResumeRequest {
    checkpoint: Option<SubscriberCheckpoint>,
    max_commits: usize,
    subscriber_contract: SubscriberContractDeclaration,
}

impl SubscriberResumeRequest {
    pub fn from_head(max_commits: usize) -> Self {
        Self {
            checkpoint: None,
            max_commits,
            subscriber_contract: SubscriberContractDeclaration::default(),
        }
    }

    pub fn resume_after(checkpoint: SubscriberCheckpoint, max_commits: usize) -> Self {
        Self {
            checkpoint: Some(checkpoint),
            max_commits,
            subscriber_contract: SubscriberContractDeclaration::default(),
        }
    }

    pub fn with_subscriber_contract(
        mut self,
        subscriber_contract: SubscriberContractDeclaration,
    ) -> Self {
        self.subscriber_contract = subscriber_contract;
        self
    }

    pub fn checkpoint(&self) -> Option<&SubscriberCheckpoint> {
        self.checkpoint.as_ref()
    }

    pub fn max_commits(&self) -> usize {
        self.max_commits
    }

    pub fn subscriber_contract(&self) -> &SubscriberContractDeclaration {
        &self.subscriber_contract
    }
}

impl Default for SubscriberResumeRequest {
    fn default() -> Self {
        Self::from_head(usize::MAX)
    }
}
