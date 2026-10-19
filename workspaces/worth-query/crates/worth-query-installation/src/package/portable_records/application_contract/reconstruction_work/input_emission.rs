//! Bound whole-input effect metadata is part of cold reconstruction work.
use super::WorthQueryPortableApplicationContractReconstructionWork;
use worth_query_declaration::facade::application_schema::ApplicationOperationBodyInputEmission;
impl WorthQueryPortableApplicationContractReconstructionWork {
    pub(super) fn observe_input_emission(
        &mut self,
        emission: &ApplicationOperationBodyInputEmission,
    ) {
        for text in [
            emission.effect.as_str(),
            emission.input_binding.as_str(),
            emission.payload_binding.as_str(),
            emission.protocol.identity().as_str(),
        ] {
            self.observe_text(text);
        }
        self.observe_fixed(12);
    }
}
