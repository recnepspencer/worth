//! One controlled compiler for ordinary and Home invariant inventories.
use super::*;
use crate::schema::{data::RelationalSchemaRegistry, SchemaCompilationControl};
use crate::validation::candidate_source::ManagedIndexSourceProgram;

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
        let mut model_ownership_index = None;
        for (slot, registration) in registrations.iter().enumerate() {
            control.checkpoint().map_err(Denial::Execution)?;
            if registration.model_ownership.is_some() {
                model_ownership_index = Some(slot);
            }
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
        let mut points: [Vec<usize>; InvariantExecutionPoint::COUNT] =
            std::array::from_fn(|point| {
                Vec::with_capacity(
                    registrations
                        .iter()
                        .filter(|registration| includes(point, registration.execution_point()))
                        .count(),
                )
            });
        for (point, slots) in points.iter_mut().enumerate() {
            for (slot, registration) in registrations.iter().enumerate() {
                control.checkpoint().map_err(Denial::Execution)?;
                if includes(point, registration.execution_point()) {
                    slots.push(slot);
                }
            }
        }
        let result = Self {
            points: Arc::new(points),
            model_ownership_index,
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

fn includes(point: usize, actual: InvariantExecutionPoint) -> bool {
    point == actual as usize
        || (point == InvariantExecutionPoint::ProposedState as usize
            && InvariantExecutionPoint::ProposedState.includes(actual))
}
