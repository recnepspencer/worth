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
    pub(super) fn into_origin(mut self) -> WorldCheckpointOrigin {
        let node = self.values.pop().expect("one prepaid recipe slot");
        assert!(
            node.next.is_none(),
            "a newly admitted branch has no successor links"
        );
        match node.evidence {
            OccurrenceEvidence::Held(origin) => origin,
            OccurrenceEvidence::Prepared => unreachable!("only an admitted origin is retained"),
        }
    }
    pub(crate) fn needs_binding(&self) -> bool {
        match &self.values[0].evidence {
            OccurrenceEvidence::Prepared => true,
            OccurrenceEvidence::Held(_) => false,
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
    retired: Option<PreparedCheckpointOccurrence>,
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
    pub(super) fn advance_retirement(
        &mut self,
        request: &ExecutionRequest<'_, '_>,
        native: &worth_relational::facade::runtime::RelationalRuntime,
    ) -> Result<bool, crate::lifecycle::RuntimeWorldCloseDenial> {
        while let Some(prepared) = self.first.as_mut() {
            request
                .consult()
                .map_err(crate::lifecycle::RuntimeWorldCloseDenial::Execution)?;
            let node = &mut prepared.values[0];
            match &mut node.evidence {
                OccurrenceEvidence::Held(origin) => {
                    if !origin.advance_retirement(request, native)? {
                        return Ok(false);
                    }
                }
                OccurrenceEvidence::Prepared => unreachable!("only a held recipe can retire"),
            }
            // Move one original prepaid link after completion. A retry starts
            // at the first unfinished occurrence, regardless of prefix length.
            let mut completed = self.first.take().expect("completed first occurrence");
            self.first = completed.values[0].next.take();
            completed.values[0].next = self.retired.take();
            self.retired = Some(completed);
        }
        Ok(true)
    }
    pub(super) fn retired_keys(&self) -> impl Iterator<Item = &RuntimeWorldRetentionKey> {
        let mut next = self.retired.as_ref();
        std::iter::from_fn(move || {
            let prepared = next?;
            let node = &prepared.values[0];
            next = node.next.as_ref();
            match &node.evidence {
                OccurrenceEvidence::Held(origin) if origin.complete => Some(origin.keys.iter()),
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
        for chain in [self.first.take(), self.retired.take()] {
            let mut next = chain;
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
}
