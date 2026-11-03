//! Each durable occurrence recipe holds the same exact successor image pair.
use super::WorldCheckpointOrigin;
use crate::basis::AdmittedCompositeRuntimeWorldBasis;
use crate::durability::WorldDurableMediumDenial as Denial;
use crate::inspection::RuntimeWorldRetentionKey;
use crate::retention::HistoryRetentionObligation;
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest};

#[derive(Debug)]
enum OccurrenceEvidence {
    Prepared,
    Held(WorldCheckpointOrigin),
    Retired([RuntimeWorldRetentionKey; 2]),
}
#[derive(Debug)]
struct OccurrenceNode {
    evidence: OccurrenceEvidence,
    next: Option<PreparedCheckpointOccurrence>,
}

/// Its one slot and reservations are prepared before Native effects.
#[derive(Debug)]
pub(crate) struct PreparedCheckpointOccurrence {
    values: Vec<OccurrenceNode>,
    _funding: [Option<ExecutionMemoryReservation>; 2],
}
impl PreparedCheckpointOccurrence {
    pub(crate) fn prepare(request: &ExecutionRequest<'_, '_>) -> Result<Self, Denial> {
        let size = std::mem::size_of::<OccurrenceNode>() as u64;
        let policy = request.allocation_policy();
        let funding = policy.reserve_memory(size).map_err(Denial::Allocation)?;
        let mut values = Vec::with_capacity(1);
        let actual = (values.capacity() as u64)
            .checked_mul(size)
            .ok_or(Denial::CheckpointOriginRetention)?;
        let surplus = if actual > size {
            policy
                .reserve_memory(actual - size)
                .map_err(Denial::Allocation)?
        } else {
            None
        };
        values.push(OccurrenceNode {
            evidence: OccurrenceEvidence::Prepared,
            next: None,
        });
        Ok(Self {
            values,
            _funding: [funding, surplus],
        })
    }
    pub(crate) fn needs_binding(&self) -> bool {
        match &self.values[0].evidence {
            OccurrenceEvidence::Prepared => true,
            OccurrenceEvidence::Held(_) => false,
            OccurrenceEvidence::Retired(_) => unreachable!("attempt custody has not retired"),
        }
    }
    pub(crate) fn bind(
        &mut self,
        basis: &AdmittedCompositeRuntimeWorldBasis,
        pins: HistoryRetentionObligation,
    ) {
        assert!(matches!(
            self.values[0].evidence,
            OccurrenceEvidence::Prepared
        ));
        self.values[0].evidence = OccurrenceEvidence::Held(WorldCheckpointOrigin::new(basis, pins));
    }
}

#[derive(Debug, Default)]
pub(super) struct CheckpointOccurrences {
    first: Option<PreparedCheckpointOccurrence>,
}
impl CheckpointOccurrences {
    pub(super) fn retain(&mut self, mut prepared: PreparedCheckpointOccurrence) {
        assert!(matches!(
            prepared.values[0].evidence,
            OccurrenceEvidence::Held(_)
        ));
        prepared.values[0].next = self.first.take();
        self.first = Some(prepared);
    }
    pub(super) fn retire(&mut self) {
        let mut next = self.first.as_mut();
        while let Some(prepared) = next {
            let node = &mut prepared.values[0];
            let OccurrenceEvidence::Held(origin) =
                std::mem::replace(&mut node.evidence, OccurrenceEvidence::Prepared)
            else {
                unreachable!("only a held recipe can retire");
            };
            let WorldCheckpointOrigin {
                basis,
                pins,
                occurrences,
            } = origin;
            assert!(occurrences.first.is_none());
            let keys = pins.retention_keys();
            drop(pins);
            drop(basis);
            node.evidence = OccurrenceEvidence::Retired(keys);
            next = node.next.as_mut();
        }
    }
    pub(super) fn retired_keys(&self) -> impl Iterator<Item = &RuntimeWorldRetentionKey> {
        let mut next = self.first.as_ref();
        std::iter::from_fn(move || {
            let prepared = next?;
            let node = &prepared.values[0];
            next = node.next.as_ref();
            match &node.evidence {
                OccurrenceEvidence::Retired(keys) => Some(keys.iter()),
                OccurrenceEvidence::Prepared | OccurrenceEvidence::Held(_) => {
                    unreachable!("retirement exposes released keys only")
                }
            }
        })
        .flatten()
    }
}
impl Drop for CheckpointOccurrences {
    fn drop(&mut self) {
        let mut next = self.first.take();
        while let Some(mut prepared) = next {
            let node = prepared
                .values
                .pop()
                .expect("one originally funded occurrence slot");
            next = node.next;
            drop(node.evidence);
        }
    }
}
