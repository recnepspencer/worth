//! Declared Head operations preserve native typed failure for each subject.
use super::*;
use crate::storage::data::RelationalStorageSubject;
#[derive(Debug, Clone, Copy)]
enum Subject {
    Head,
    Symbols,
}

#[cfg(test)]
#[path = "cold_current_failure_matrix/exhaustive.rs"]
mod exhaustive;
