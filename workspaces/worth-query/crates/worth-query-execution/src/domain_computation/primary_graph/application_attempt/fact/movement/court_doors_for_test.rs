//! Sealed test doors over this module's private members.
use super::*;
use crate::domain_computation::primary_graph::vocabulary::application_attempt::movement::Movement;

impl FactMovement {
    #[doc(hidden)]
    pub fn movement_for_test(self) -> Movement {
        Self::movement(self)
    }
}
