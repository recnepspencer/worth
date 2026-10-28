//! Cold, validated interpretation attached only to its actual catalog binding.
use super::{Denial, Kind};
use crate::{
    application_query::binding::WorthQueryCompiledApplicationQueryBinding,
    application_schema::WorthQueryInstalledApplicationSchema,
    graph_obligation::{
        bind_inspection_obligations, install_disclosure_capabilities,
        WorthQueryInstalledGraphObligationSet,
    },
};
use std::sync::Arc;
use worth_query_declaration::facade::{
    application_query::{
        ApplicationQueryDisclosurePosture, ApplicationQueryInspectionMetadataFacet,
        ApplicationQueryInspectionMetadataMeaning, ApplicationQueryResultSlotKey,
        ErasedApplicationQueryInspectionDefinition,
    },
    application_schema::ApplicationSchema,
};

pub(crate) struct CompiledInspectionMeaning {
    pub(crate) definition: ErasedApplicationQueryInspectionDefinition,
    pub(crate) slots: [ApplicationQueryResultSlotKey; 6],
    pub(crate) obligations: Arc<WorthQueryInstalledGraphObligationSet>,
}

impl CompiledInspectionMeaning {
    /// The catalog has already validated every original binding and definition.
    pub(in crate::application_query) fn compile<Schema: ApplicationSchema>(
        schema: &WorthQueryInstalledApplicationSchema<Schema>,
        binding: &WorthQueryCompiledApplicationQueryBinding,
        definition: &ErasedApplicationQueryInspectionDefinition,
    ) -> Result<Self, Denial> {
        let contract = &definition.parts().metadata_disclosure;
        let capabilities = install_disclosure_capabilities(schema, contract);
        let name = binding.descriptor().identity();
        if contract.posture() == ApplicationQueryDisclosurePosture::Governed
            && capabilities.is_empty()
        {
            return Err(Denial::new(Kind::InvalidInspectionDefinition, name));
        }
        let query = binding.query();
        let obligations = bind_inspection_obligations(
            &schema.binding_identity(),
            name,
            definition,
            query.authorization(),
            &capabilities,
        )
        .map_err(|cause| crate::application_query::denial::graph_obligation_denial(name, cause))?;
        let slots = ApplicationQueryInspectionMetadataFacet::ALL.map(|facet| {
            ApplicationQueryInspectionMetadataMeaning::from_untrusted_fields(
                definition.parts().binding.clone(),
                facet,
            )
            .slot_key()
        });
        Ok(Self {
            definition: definition.clone(),
            slots,
            obligations: Arc::new(obligations),
        })
    }
}
