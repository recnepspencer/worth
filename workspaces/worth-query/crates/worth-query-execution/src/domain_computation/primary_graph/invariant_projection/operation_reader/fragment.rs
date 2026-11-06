//! Fragment reads retain native snapshot reads, decision facts and original controls.

use super::super::super::fragment_view::{ErasedObserved, FragmentDenial, ReadPort, Reading, View};
use super::super::super::HandlerExecutionDenial;
use super::WorthQueryApplicationOperationInvariantProjectionReader as Reader;
use std::collections::BTreeMap;
use worth_query_declaration::facade::application_schema::fragment::{AspectDefinition, Fragment};
use worth_query_installation::facade::ApplicationSchema;

impl<S: ApplicationSchema, O> Reader<'_, '_, S, O> {
    pub fn fragment<F: Fragment>(
        &mut self,
        marker: F,
    ) -> Result<View<'_, F, Reading<'_>>, FragmentDenial> {
        let catalog = self.reader.value_bindings;
        let definition = catalog
            .fragment(marker)
            .ok_or(FragmentDenial::NotInstalled { fragment: F::NAME })?;
        Ok(View::reading(definition, self))
    }
}

impl<S: ApplicationSchema, O> ReadPort for Reader<'_, '_, S, O> {
    fn with_decode_admission(
        &mut self,
        path: &str,
        decode: &mut dyn FnMut(
            &mut worth_query_declaration::facade::application_schema::ValueDecodeAdmission<'_>,
        ) -> Result<(), FragmentDenial>,
    ) -> Result<(), FragmentDenial> {
        super::fragment_native::with_decode_admission(self.reader.value_decode, path, decode)
    }

    fn resolve(
        &mut self,
        field: &worth_query_declaration::facade::application_schema::ApplicationFieldBindingLocus,
        value: worth_foundational::facade::AspectValue,
    ) -> Result<ErasedObserved, FragmentDenial> {
        self.native_fragment().resolve(field, value)
    }

    fn record(
        &mut self,
        target: &ErasedObserved,
        aspect: &AspectDefinition,
    ) -> Result<Option<BTreeMap<String, worth_foundational::facade::AspectValue>>, FragmentDenial>
    {
        self.native_fragment().record(target, aspect)
    }

    fn target(
        &self,
        target: &ErasedObserved,
    ) -> Result<super::super::WorthQueryInvariantMutationTarget<(), ()>, FragmentDenial> {
        super::mutation_target::target(
            target.identity.native_identity(),
            self.reader.authority_identity,
            self.runtime_authority,
            self.binding_identity,
            self.admission_identity,
        )
        .map_err(|cause| FragmentDenial::Read(HandlerExecutionDenial::new(cause)))
    }
}
