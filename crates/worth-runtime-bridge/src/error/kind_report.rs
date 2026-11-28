use std::error::Error;

use super::*;
use worth_execution::report::{DenialClass, Disposition};

pub(super) trait BridgeErrorKind: std::fmt::Debug {
    fn report(&self) -> (DenialClass, Disposition, &'static str);
    fn summary(&self) -> &'static str;
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

mod delivery;
mod lifecycle;
mod planning;
mod replay;
