//! Compile one genuinely supplied Home target under the Native root issuer.
use super::*;

impl RelationalBranchRootSchemaAuthority {
    pub(crate) fn prepare_home_target(
        request: &worth_execution::ExecutionRequest<'_, '_>,
        issuer: &super::super::RelationalBranchRootIdentityIssuer,
        registry: RelationalSchemaRegistry,
        descriptor: crate::schema::data::DescriptorSemanticsVersion,
        previous: &Self,
    ) -> Result<Arc<Self>, super::super::RelationalBranchRootCaptureDenial> {
        let control = crate::schema::SchemaCompilationControl::Request(*request);
        let quote =
            compilation_quote::SchemaCompilationQuote::for_target(&registry, previous, control)?;
        let (memory, temporary) = quote.reserve(request)?;
        crate::execution::check_request(request)
            .map_err(super::super::RelationalBranchRootCaptureDenial::Request)?;
        let expected = registry
            .snapshot_for_compilation(control)
            .map_err(super::super::RelationalBranchRootCaptureDenial::Request)?;
        let id = issuer.issue_schema_authority_id()?;
        let funding = Arc::new(compiled_retention::SchemaCompilationFunding { _memory: memory });
        let schema = Self::compile_owned(
            id,
            registry,
            &expected,
            descriptor,
            compilation::RetainedContracts::Extend(Some(previous)),
            Some(funding),
            control,
        )
        .map_err(super::super::RelationalBranchRootCaptureDenial::Request)?;
        // Descriptor and deduplication scratch drop before their original grant.
        drop(expected);
        drop(temporary);
        Ok(schema)
    }
}

#[cfg(test)]
#[path = "home_target/tests.rs"]
mod tests;
