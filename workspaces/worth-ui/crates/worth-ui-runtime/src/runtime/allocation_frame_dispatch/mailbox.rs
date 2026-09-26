use std::ops::Index;

use super::UiAdmittedAllocationStreamIngress;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiAllocationFrameMailboxStoragePosture {
    admitted_capacity: u16,
    inline_slot_count: u16,
}

/// Allocation-free, fixed-footprint transport storage owned for one dispatcher lifetime.
#[derive(Debug)]
pub(crate) struct UiAllocationFrameMailbox {
    capacity: u16,
    len: u16,
    ingress: [Option<UiAdmittedAllocationStreamIngress>; super::ALLOCATION_FRAME_SOURCE_CAPACITY],
}

/// Move-only canonical mailbox contents transferred into a sealed or terminal outcome.
#[derive(Debug)]
pub(crate) struct UiAllocationFrameMailboxDrain {
    len: u16,
    ingress: [Option<UiAdmittedAllocationStreamIngress>; super::ALLOCATION_FRAME_SOURCE_CAPACITY],
}

#[derive(Clone, Copy, Debug)]
pub struct UiAllocationFrameIngressView<'a> {
    ingress: &'a [Option<UiAdmittedAllocationStreamIngress>],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UiAllocationFrameMailboxInsertWork {
    pub(crate) comparisons: u64,
    pub(crate) canonical_writes: u64,
}

fn empty_slots(
) -> [Option<UiAdmittedAllocationStreamIngress>; super::ALLOCATION_FRAME_SOURCE_CAPACITY] {
    std::array::from_fn(|_| None)
}

impl UiAllocationFrameMailbox {
    pub(crate) fn new(capacity: u16) -> Self {
        assert!(usize::from(capacity) <= super::ALLOCATION_FRAME_SOURCE_CAPACITY);
        Self {
            capacity,
            len: 0,
            ingress: empty_slots(),
        }
    }

    pub(crate) fn capacity(&self) -> u16 {
        self.capacity
    }

    pub(crate) fn storage_posture(&self) -> UiAllocationFrameMailboxStoragePosture {
        UiAllocationFrameMailboxStoragePosture {
            admitted_capacity: self.capacity,
            inline_slot_count: super::ALLOCATION_FRAME_SOURCE_COUNT,
        }
    }

    pub(crate) fn len(&self) -> u16 {
        self.len
    }

    pub(crate) fn is_full(&self) -> bool {
        self.len >= self.capacity
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub(crate) fn insert(
        &mut self,
        ingress: UiAdmittedAllocationStreamIngress,
    ) -> UiAllocationFrameMailboxInsertWork {
        self.ingress[usize::from(self.len)] = Some(ingress);
        self.len += 1;
        UiAllocationFrameMailboxInsertWork {
            comparisons: 0,
            canonical_writes: 0,
        }
    }

    pub(crate) fn drain_canonical(
        &mut self,
    ) -> (
        UiAllocationFrameMailboxDrain,
        UiAllocationFrameMailboxInsertWork,
    ) {
        let len = usize::from(self.len);
        let mut arrival = std::mem::replace(&mut self.ingress, empty_slots());
        let mut ranks = [0_usize; super::ALLOCATION_FRAME_SOURCE_CAPACITY];
        let mut comparisons = 0;
        for candidate_index in 0..len {
            let candidate = arrival[candidate_index]
                .as_ref()
                .expect("active mailbox slots are populated");
            for comparison in arrival.iter().take(len) {
                comparisons += 1;
                if canonical_order(
                    comparison
                        .as_ref()
                        .expect("active mailbox slots are populated"),
                ) < canonical_order(candidate)
                {
                    ranks[candidate_index] += 1;
                }
            }
        }
        let mut canonical = empty_slots();
        for index in 0..len {
            canonical[ranks[index]] = arrival[index].take();
        }
        let drain = UiAllocationFrameMailboxDrain {
            len: self.len,
            ingress: canonical,
        };
        self.len = 0;
        (
            drain,
            UiAllocationFrameMailboxInsertWork {
                comparisons,
                canonical_writes: len as u64,
            },
        )
    }
}

fn canonical_order(
    ingress: &UiAdmittedAllocationStreamIngress,
) -> (
    super::UiAllocationFrameSourceLane,
    super::UiAllocationFrameSourceIdentity,
    super::UiAllocationFrameSourceGeneration,
    super::UiAdmittedAllocationSourceOrder,
    super::UiAllocationFrameIngressIdentity,
) {
    (
        ingress.source_lane(),
        ingress.source_identity(),
        ingress.source_generation(),
        ingress.source_order(),
        ingress.identity(),
    )
}

impl UiAllocationFrameMailboxDrain {
    pub(crate) fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub(crate) fn view(&self) -> UiAllocationFrameIngressView<'_> {
        UiAllocationFrameIngressView {
            ingress: &self.ingress[..usize::from(self.len)],
        }
    }

    pub(crate) fn iter(&self) -> impl ExactSizeIterator<Item = &UiAdmittedAllocationStreamIngress> {
        self.view().iter()
    }

    pub(crate) fn into_ingress(self) -> Box<[UiAdmittedAllocationStreamIngress]> {
        self.ingress
            .into_iter()
            .take(usize::from(self.len))
            .map(|slot| slot.expect("active mailbox slots are populated"))
            .collect::<Vec<_>>()
            .into_boxed_slice()
    }
}

impl PartialEq for UiAllocationFrameMailboxDrain {
    fn eq(&self, other: &Self) -> bool {
        self.view() == other.view()
    }
}

impl<'a> UiAllocationFrameIngressView<'a> {
    pub fn len(self) -> usize {
        self.ingress.len()
    }

    #[cfg(test)]
    pub fn is_empty(self) -> bool {
        self.ingress.is_empty()
    }

    pub fn iter(self) -> impl ExactSizeIterator<Item = &'a UiAdmittedAllocationStreamIngress> {
        self.ingress
            .iter()
            .map(|slot| slot.as_ref().expect("visible ingress slots are populated"))
    }
}

impl Index<usize> for UiAllocationFrameIngressView<'_> {
    type Output = UiAdmittedAllocationStreamIngress;

    fn index(&self, index: usize) -> &Self::Output {
        self.ingress[index]
            .as_ref()
            .expect("visible ingress slots are populated")
    }
}

impl PartialEq for UiAllocationFrameIngressView<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.iter().eq(other.iter())
    }
}

impl<const N: usize> PartialEq<&[UiAdmittedAllocationStreamIngress; N]>
    for UiAllocationFrameIngressView<'_>
{
    fn eq(&self, other: &&[UiAdmittedAllocationStreamIngress; N]) -> bool {
        self.iter().eq(other.iter())
    }
}

impl UiAllocationFrameMailboxStoragePosture {
    pub fn admitted_capacity(self) -> u16 {
        self.admitted_capacity
    }

    pub fn inline_slot_count(self) -> u16 {
        self.inline_slot_count
    }
}
