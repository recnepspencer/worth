//! Sealed test doors over this module's private members.
use super::*;

impl WorthQueryApplicationRichMaterialization {
    #[doc(hidden)]
    pub fn request_for_test(&self) -> &WorthQueryApplicationMaterializationRequest {
        &self.request
    }
}

impl WorthQueryApplicationRichMaterialization {
    #[doc(hidden)]
    pub fn request_mut_for_test(&mut self) -> &mut WorthQueryApplicationMaterializationRequest {
        &mut self.request
    }
}
