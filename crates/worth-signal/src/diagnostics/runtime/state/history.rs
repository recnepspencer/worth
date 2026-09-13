//! Persistent diagnostic ordering with immutable shared frames.
use std::sync::{Arc, OnceLock};

#[cfg(test)]
mod cold_preparation_tests;
mod retained_charge;
mod work_admission;
use crate::data::retained_storage::{ordered_index_charge, RetainedStorageCharge};
pub(crate) use retained_charge::DiagnosticHistoryEditDenial;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DiagnosticHistoryPositionExhausted;

/// Positions are private storage order, never replay or lineage authority.
/// Cloning shares tree roots and frames; eviction cannot release a sibling's
/// retained frame. Slot/draft custody must account for both roots separately.
#[derive(Debug)]
pub(crate) struct DiagnosticHistory<T> {
    entries: im::OrdMap<u64, Arc<T>>,
    next_position: Option<u64>,
    // Nonsemantic cold-preparation metadata; never shared separately from this value.
    retained_charge: OnceLock<RetainedStorageCharge>,
}

impl<T> Clone for DiagnosticHistory<T> {
    fn clone(&self) -> Self {
        Self {
            entries: self.entries.clone(),
            next_position: self.next_position,
            retained_charge: self.retained_charge.clone(),
        }
    }
}

impl<T> Default for DiagnosticHistory<T> {
    fn default() -> Self {
        Self {
            entries: im::OrdMap::new(),
            next_position: Some(0),
            retained_charge: ordered_index_charge::<u64, Arc<T>>(0)
                .map(OnceLock::from)
                .unwrap_or_default(),
        }
    }
}

impl<T> DiagnosticHistory<T> {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn len(&self) -> usize {
        self.entries.len()
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub(crate) fn iter(&self) -> DiagnosticHistoryIter<'_, T> {
        DiagnosticHistoryIter {
            entries: Some(self.entries.iter()),
        }
    }

    pub(crate) fn front(&self) -> Option<&T> {
        self.iter().next()
    }

    pub(crate) fn back(&self) -> Option<&T> {
        self.iter().next_back()
    }

    pub(crate) fn push_back(&mut self, value: T) -> Result<(), DiagnosticHistoryPositionExhausted> {
        let position = self
            .next_position
            .ok_or(DiagnosticHistoryPositionExhausted)?;
        let _ = self.retained_charge.take();
        self.entries.insert(position, Arc::new(value));
        self.next_position = position.checked_add(1);
        Ok(())
    }

    pub(crate) fn pop_front(&mut self) -> Option<Arc<T>> {
        let position = *self.entries.keys().next()?;
        let _ = self.retained_charge.take();
        self.entries.remove(&position)
    }

    /// The caller has already located this ordinal by inspecting its history.
    /// This is a positional operation, not a claim of constant traversal cost.
    pub(crate) fn remove(&mut self, index: usize) -> Option<Arc<T>> {
        let position = *self.entries.keys().nth(index)?;
        let _ = self.retained_charge.take();
        self.entries.remove(&position)
    }

    pub(crate) fn clear(&mut self) {
        let _ = self.retained_charge.take();
        self.entries.clear();
        self.retained_charge = ordered_index_charge::<u64, Arc<T>>(0)
            .map(OnceLock::from)
            .unwrap_or_default();
    }
}

pub(crate) struct DiagnosticHistoryIter<'a, T> {
    entries: Option<im::ordmap::Iter<'a, u64, Arc<T>>>,
}

impl<T> Default for DiagnosticHistoryIter<'_, T> {
    fn default() -> Self {
        Self { entries: None }
    }
}

impl<'a, T> Iterator for DiagnosticHistoryIter<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<Self::Item> {
        self.entries
            .as_mut()?
            .next()
            .map(|(_, frame)| frame.as_ref())
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.entries.as_ref().map_or(0, ExactSizeIterator::len);
        (len, Some(len))
    }
}

impl<T> DoubleEndedIterator for DiagnosticHistoryIter<'_, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.entries
            .as_mut()?
            .next_back()
            .map(|(_, frame)| frame.as_ref())
    }
}

impl<T> ExactSizeIterator for DiagnosticHistoryIter<'_, T> {}

impl<'a, T> IntoIterator for &'a DiagnosticHistory<T> {
    type Item = &'a T;
    type IntoIter = DiagnosticHistoryIter<'a, T>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<T: PartialEq> PartialEq for DiagnosticHistory<T> {
    fn eq(&self, other: &Self) -> bool {
        self.len() == other.len() && self.iter().eq(other.iter())
    }
}

impl<T: Serialize> Serialize for DiagnosticHistory<T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_seq(self.iter())
    }
}

impl<'de, T: Deserialize<'de>> Deserialize<'de> for DiagnosticHistory<T> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let values = Vec::<T>::deserialize(deserializer)?;
        let mut history = Self::new();
        for value in values {
            history.push_back(value).map_err(|_| {
                serde::de::Error::custom("diagnostic history position space exhausted")
            })?;
        }
        Ok(history)
    }
}

impl<T> FromIterator<T> for DiagnosticHistory<T> {
    fn from_iter<I: IntoIterator<Item = T>>(values: I) -> Self {
        let mut history = Self::new();
        for value in values {
            history
                .push_back(value)
                .expect("materialized history fits its position space");
        }
        history
    }
}

#[cfg(test)]
#[path = "history_tests.rs"]
mod tests;
