//! Exact cold compiler admission retains both descriptor and compiled backing.
use super::*;
struct ExactSchemaInputs {
    registry: RelationalSchemaRegistry,
    contracts: Vec<AspectContract>,
}
impl RelationalBranchRootSchemaAuthority {
    pub(crate) fn readmit_exact(
        allocation_id: u64,
        registry: RelationalSchemaRegistry,
        retained_aspect_contracts: Vec<AspectContract>,
        expected: &SchemaAuthoritySnapshot,
        descriptor_semantics_version: crate::schema::data::DescriptorSemanticsVersion,
        control: crate::schema::SchemaCompilationControl<'_, '_>,
    ) -> Result<Option<Arc<Self>>, crate::execution::RelationalExecutionDenialCause> {
        let (funding, temporary) = match control {
            crate::schema::SchemaCompilationControl::Configuration => (None, None),
            crate::schema::SchemaCompilationControl::Request(request) => {
                let quote = compilation_quote::SchemaCompilationQuote::for_exact(
                    &registry,
                    &retained_aspect_contracts,
                    control,
                )
                .map_err(crate::execution::RelationalExecutionDenialCause::from)?;
                let (memory, temporary) = quote
                    .reserve(&request)
                    .map_err(crate::execution::RelationalExecutionDenialCause::from)?;
                (
                    Some(Arc::new(compiled_retention::SchemaCompilationFunding {
                        _memory: memory,
                    })),
                    temporary,
                )
            }
        };
        // Inputs clean up before their adopted compiler grant on every exit.
        let inputs = ExactSchemaInputs {
            registry,
            contracts: retained_aspect_contracts,
        };
        let observed = inputs.registry.snapshot_for_compilation(control)?;
        if observed != *expected {
            return Ok(None);
        }
        // The caller-local hold covers input cleanup on every compiler error.
        let compiled = Self::compile_owned(
            allocation_id,
            inputs.registry,
            &observed,
            descriptor_semantics_version,
            compilation::RetainedContracts::Exact(inputs.contracts),
            funding.clone(),
            control,
        );
        drop(temporary);
        drop(funding);
        compiled.map(Some)
    }
}

#[cfg(test)]
#[path = "readmission/tests.rs"]
mod tests;
