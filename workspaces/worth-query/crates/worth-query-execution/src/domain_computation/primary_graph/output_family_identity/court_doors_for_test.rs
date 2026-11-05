//! Sealed test doors over this module's private members.
use super::*;
use crate::domain_computation::primary_graph::vocabulary::output_family_identity::OutputFamilyIdentity;


impl OutputFamilyIdentity {
    #[doc(hidden)]
    pub fn as_str_for_test(&self) -> &str {
        Self::as_str(self)
    }
}
