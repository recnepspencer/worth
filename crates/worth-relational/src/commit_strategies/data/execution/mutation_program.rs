use std::sync::Arc;

use serde::{de::Error as DeError, Deserialize, Deserializer, Serialize};

use crate::transactions::data::WorkerIntentBatch;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StrategyMutationProgramDigest(pub [u8; 32]);

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct StrategyMutationProgram {
    worker_batches: Arc<[WorkerIntentBatch]>,
    digest: StrategyMutationProgramDigest,
    total_intent_count: usize,
}

impl StrategyMutationProgram {
    pub fn new(worker_batches: impl Into<Arc<[WorkerIntentBatch]>>) -> Self {
        let worker_batches = worker_batches.into();
        let digest = compute_mutation_program_digest(&worker_batches);
        let total_intent_count = worker_batches.iter().map(|batch| batch.intents.len()).sum();
        Self {
            worker_batches,
            digest,
            total_intent_count,
        }
    }

    pub fn worker_batches(&self) -> &[WorkerIntentBatch] {
        &self.worker_batches
    }

    pub fn digest(&self) -> StrategyMutationProgramDigest {
        self.digest
    }

    pub fn total_intent_count(&self) -> usize {
        self.total_intent_count
    }
}

impl<'de> Deserialize<'de> for StrategyMutationProgram {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct RawStrategyMutationProgram {
            worker_batches: Arc<[WorkerIntentBatch]>,
            digest: StrategyMutationProgramDigest,
            total_intent_count: usize,
        }

        let raw = RawStrategyMutationProgram::deserialize(deserializer)?;
        let expected_digest = compute_mutation_program_digest(&raw.worker_batches);
        if raw.digest != expected_digest {
            return Err(D::Error::custom(
                "strategy mutation program digest does not match canonical worker batches",
            ));
        }
        let expected_total_intent_count: usize = raw
            .worker_batches
            .iter()
            .map(|batch| batch.intents.len())
            .sum();
        if raw.total_intent_count != expected_total_intent_count {
            return Err(D::Error::custom(
                "strategy mutation program intent count does not match canonical worker batches",
            ));
        }
        Ok(Self {
            worker_batches: raw.worker_batches,
            digest: raw.digest,
            total_intent_count: raw.total_intent_count,
        })
    }
}

pub(super) fn compute_mutation_program_digest(
    worker_batches: &[WorkerIntentBatch],
) -> StrategyMutationProgramDigest {
    super::super::strategy_mutation_program_digest(worker_batches)
}

#[cfg(test)]
pub(super) fn forged_mutation_program_for_digest_test(
    program: &StrategyMutationProgram,
    digest: StrategyMutationProgramDigest,
) -> StrategyMutationProgram {
    StrategyMutationProgram {
        worker_batches: Arc::from(program.worker_batches()),
        digest,
        total_intent_count: program.total_intent_count(),
    }
}

impl Serialize for StrategyMutationProgramDigest {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        worth_foundational::CanonicalDigestId::new(self.0).serialize(serializer)
    }
}
impl<'de> Deserialize<'de> for StrategyMutationProgramDigest {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        worth_foundational::CanonicalDigestId::deserialize(deserializer)
            .map(|digest| Self(*digest.bytes()))
    }
}
