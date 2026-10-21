//! Bound whole-input effect metadata is part of cold reconstruction work.
use super::WorthQueryPortableApplicationContractReconstructionWork;
use worth_query_declaration::facade::application_schema::ApplicationOperationBodyExternalEmission;
impl WorthQueryPortableApplicationContractReconstructionWork {
    pub(super) fn observe_external_emission(
        &mut self,
        emission: &ApplicationOperationBodyExternalEmission,
    ) {
        if let worth_query_declaration::facade::application_schema::ApplicationOperationBodyExternalPayloadSource::QueryContext { mutation_binding, query }
            | worth_query_declaration::facade::application_schema::ApplicationOperationBodyExternalPayloadSource::Evaluation { mutation_binding, query } = &emission.source {
            self.observe_query_context(mutation_binding, query);
        }
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
