//! The HTTP listener's close result preserves installed rail custody authority.

use std::fmt;
use std::io;

use super::inbound_completion::BankRailCompletionClose;

pub struct BankHttpServerClose {
    rail_completion: Option<BankRailCompletionClose>,
}

pub struct BankHttpServerCloseFailure {
    error: io::Error,
    close: BankHttpServerClose,
}

impl BankHttpServerClose {
    pub(super) fn new(rail_completion: Option<BankRailCompletionClose>) -> Self {
        Self { rail_completion }
    }

    pub fn rail_completion(&self) -> Option<&BankRailCompletionClose> {
        self.rail_completion.as_ref()
    }

    pub fn into_rail_completion(self) -> Option<BankRailCompletionClose> {
        self.rail_completion
    }
}

impl BankHttpServerCloseFailure {
    pub(super) fn new(error: io::Error, close: BankHttpServerClose) -> Self {
        Self { error, close }
    }

    pub fn error(&self) -> &io::Error {
        &self.error
    }

    pub fn into_close(self) -> BankHttpServerClose {
        self.close
    }
}

impl fmt::Debug for BankHttpServerCloseFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("BankHttpServerCloseFailure")
            .field("error", &self.error)
            .field(
                "rail_completion_retained",
                &self.close.rail_completion.is_some(),
            )
            .finish()
    }
}

impl fmt::Display for BankHttpServerCloseFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "Bank HTTP shutdown: {}", self.error)
    }
}

impl std::error::Error for BankHttpServerCloseFailure {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.error)
    }
}
