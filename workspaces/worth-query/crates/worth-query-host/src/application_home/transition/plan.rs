//! A complete descriptive relational body roster; Home alone grants its steps.
use worth_foundational::facade::{
    HomeProgramTransitionBodyDigest, HomeProgramTransitionPlanDigest, HomeProgramTransitionStepRole,
};
use worth_relational::facade::{
    mvcc::RelationalTransactionIntent,
    schema::{ProposedSchemaTransition, SchemaReconciliationPolicy},
    transactions::WorkerIntentBatch,
};

pub struct WorthQueryHomeProgramTransitionStep {
    pub batches: Vec<WorkerIntentBatch>,
}
pub struct WorthQueryHomeProgramTransitionPlan {
    pub steps: Vec<WorthQueryHomeProgramTransitionStep>,
    pub target: u32,
    pub schema_transition: Option<(ProposedSchemaTransition, Option<SchemaReconciliationPolicy>)>,
}
impl WorthQueryHomeProgramTransitionPlan {
    pub(super) fn role(&self, position: u32) -> HomeProgramTransitionStepRole {
        match position.cmp(&self.target) {
            std::cmp::Ordering::Less => HomeProgramTransitionStepRole::BeforeTarget,
            std::cmp::Ordering::Equal => HomeProgramTransitionStepRole::Target,
            std::cmp::Ordering::Greater => HomeProgramTransitionStepRole::AfterTarget,
        }
    }
    pub(super) fn intent(&self, position: u32) -> RelationalTransactionIntent {
        let intent = RelationalTransactionIntent::ordinary();
        match (&self.schema_transition, position == self.target) {
            (Some((schema, policy)), true) => {
                intent.with_schema_transition(schema.clone(), policy.clone())
            }
            (Some(_), false) | (None, true) | (None, false) => intent,
        }
    }
    pub(super) fn digest(
        &self,
        bodies: &[HomeProgramTransitionBodyDigest],
    ) -> HomeProgramTransitionPlanDigest {
        use sha2::{Digest, Sha256};
        let mut hash = Sha256::new();
        hash.update(b"worth.host.home-program-transition.v1");
        hash.update(self.target.to_le_bytes());
        hash.update((bodies.len() as u64).to_le_bytes());
        for body in bodies {
            hash.update(body.bytes());
        }
        HomeProgramTransitionPlanDigest::from_recorded(hash.finalize().into())
    }
}
