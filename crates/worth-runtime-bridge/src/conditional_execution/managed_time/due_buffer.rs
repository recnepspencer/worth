use super::super::retention::BridgeRetentionReservation;
use super::BridgeManagedDueWake;
use std::sync::Arc;

/// Owns both the promoted wake array and its charge, including empty capacity
/// after callers extract individual wakes. Extracted wakes carry custody too.
pub struct BridgeManagedDueWakeBuffer {
    pub(super) wakes: Vec<BridgeManagedDueWake>,
    pub(super) reservation: Arc<BridgeRetentionReservation>,
}

impl BridgeManagedDueWakeBuffer {
    pub fn pop(&mut self) -> Option<BridgeManagedDueWake> {
        self.wakes.pop()
    }
}

impl std::ops::Deref for BridgeManagedDueWakeBuffer {
    type Target = [BridgeManagedDueWake];
    fn deref(&self) -> &Self::Target {
        &self.wakes
    }
}

pub struct BridgeManagedDueWakeIntoIter {
    wakes: std::vec::IntoIter<BridgeManagedDueWake>,
    _reservation: Arc<BridgeRetentionReservation>,
}

impl IntoIterator for BridgeManagedDueWakeBuffer {
    type Item = BridgeManagedDueWake;
    type IntoIter = BridgeManagedDueWakeIntoIter;
    fn into_iter(self) -> Self::IntoIter {
        BridgeManagedDueWakeIntoIter {
            wakes: self.wakes.into_iter(),
            _reservation: self.reservation,
        }
    }
}

impl Iterator for BridgeManagedDueWakeIntoIter {
    type Item = BridgeManagedDueWake;
    fn next(&mut self) -> Option<Self::Item> {
        self.wakes.next()
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.wakes.size_hint()
    }
}

impl ExactSizeIterator for BridgeManagedDueWakeIntoIter {}
