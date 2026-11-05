//! One controlled compiler for ordinary and Home invariant inventories.
use super::*;
use crate::schema::{data::RelationalSchemaRegistry, SchemaCompilationControl};
use crate::validation::candidate_source::ManagedIndexSourceProgram;

#[derive(Debug)]
pub(crate) enum CustomInvariantCompilationDenial {
    Source(crate::derived_relation::DerivedRelationDenial),
    Duplicate(DuplicateCustomInvariantRegistration),
    Execution(crate::execution::RelationalExecutionDenialCause),
    CapacityOverflow,
}
impl CustomInvariantCompilationDenial {
    /// The descriptive Source binding path has no execution request or storage
    /// admission. Its shared validation can therefore refuse only its meaning.
    pub(crate) fn configuration_source(self) -> crate::derived_relation::DerivedRelationDenial {
        match self {
            Self::Source(cause) => cause,
            Self::Execution(_) | Self::Duplicate(_) | Self::CapacityOverflow => {
                unreachable!("configuration Source validation neither reserves nor executes")
            }
        }
    }
}
impl FrozenCustomInvariantRegistry {
    pub(crate) fn compile(
        registry: &RelationalSchemaRegistry,
        registrations: Vec<CustomInvariantRegistration>,
        public_sources: Vec<ManagedIndexSourceProgram>,
        control: SchemaCompilationControl<'_, '_>,
    ) -> Result<Self, CustomInvariantCompilationDenial> {
        let quote = compilation_quote::CustomInvariantCompilationQuote::measure(
            &registrations,
            &public_sources,
            control,
        )?;
        let (funding, temporary) = quote.reserve(control)?;
        let result = Self::compile_owned(
            registry,
            registrations,
            public_sources,
            quote,
            funding,
            control,
        )?;
        drop(temporary);
        Ok(result)
    }
    pub(super) fn compile_owned(
        registry: &RelationalSchemaRegistry,
        registrations: Vec<CustomInvariantRegistration>,
        public_sources: Vec<ManagedIndexSourceProgram>,
        quote: compilation_quote::CustomInvariantCompilationQuote,
        funding: Option<Arc<CustomInvariantCompilationFunding>>,
        control: SchemaCompilationControl<'_, '_>,
    ) -> Result<Self, CustomInvariantCompilationDenial> {
        use CustomInvariantCompilationDenial as Denial;
        // Preserve the original public-Source-before-duplicate refusal order.
        let public = SourcePreparationGroup::for_public_sources(registry, public_sources, control)?;
        let mut index = BTreeMap::new();
        for (slot, registration) in registrations.iter().enumerate() {
            control.checkpoint().map_err(Denial::Execution)?;
            let identity = registration.descriptor().identity.clone();
            let execution_point = registration.execution_point();
            if index
                .insert((identity.rule_id.clone(), execution_point), slot)
                .is_some()
            {
                return Err(Denial::Duplicate(DuplicateCustomInvariantRegistration {
                    identity,
                    execution_point,
                }));
            }
        }
        let mut groups = Vec::with_capacity(quote.groups);
        for registration in &registrations {
            control.checkpoint().map_err(Denial::Execution)?;
            if let Some(group) = SourcePreparationGroup::from_invariant(registration, control)? {
                groups.push(group);
            }
        }
        for group in public {
            control.checkpoint().map_err(Denial::Execution)?;
            groups.push(group);
        }
        let history = crate::validation::candidate_source::authored_history::catalog::SourceHistoryCatalog::compile(
            &groups, quote.programs, funding.clone(), control,
        )?;
        let postings =
            crate::validation::candidate_source::routing::SourceProgramPostings::compile(
                &groups, control,
            )?;
        let result = Self {
            registrations: registrations.into(),
            source_groups: Arc::new(SourcePreparationInventory::new(groups, funding.clone())),
            source_postings: Arc::new(postings),
            source_history: Arc::new(history),
            index_by_identity_and_execution_point: Arc::new(index),
            _funding: funding,
        };
        Ok(result)
    }
}
