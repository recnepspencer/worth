//! One canonical-prefix admission with ordinary or managed-impact coordinates.
use super::*;
use crate::history::{
    AuthoredWritePostingKey as PostingKey, RelationalAuthoredManagedSourceSelector,
    RelationalAuthoredWriteSelector as Selector,
};
use crate::validation::candidate_source::authored_history::{
    catalog::SourceHistoryCatalog, ManagedSourceImpactKey,
};
use worth_execution::{ChargedBytes, MapKernelContext};

enum Selection<'a> {
    Ordinary(&'a [Selector]),
    Managed(&'a [RelationalAuthoredManagedSourceSelector]),
}
impl Selection<'_> {
    fn ordinary(&self) -> &[Selector] {
        match self {
            Self::Ordinary(keys) => keys,
            Self::Managed(_) => &[],
        }
    }
    fn len(&self) -> usize {
        match self {
            Self::Ordinary(keys) => keys.len(),
            Self::Managed(keys) => keys.len(),
        }
    }
    fn validate(
        &self,
        runtime: &RelationalRuntime,
        context: &mut MapKernelContext<'_, '_>,
    ) -> Result<Option<Arc<SourceHistoryCatalog>>, Denial> {
        let Self::Managed(selectors) = self else {
            return Ok(None);
        };
        let current = &runtime
            .schema_contract_runtime
            .custom_invariant_registries
            .source_history;
        for selector in *selectors {
            super::super::visit(context)?;
            if !Arc::ptr_eq(current, &selector.catalog) || selector.program >= current.len() {
                return Err(Denial::ForeignManagedSourceProgram);
            }
            if selector.scope.generation_value() == 0 {
                return Err(Denial::UnsupportedManagedSourceSelector);
            }
        }
        Ok(Some(current.clone()))
    }
}
impl RelationalRuntime {
    /// Select authored writes strictly after the genuine source root within the
    /// target's canonical single-parent prefix. Missing/merge accelerators refuse;
    /// equality-index predicates also require actual installed-definition coverage
    /// throughout the open source-to-target interval. No implicit rebuild occurs.
    pub fn authored_write_history_cursor(
        &self,
        execution: ExecutionRequest<'_, '_>,
        original: &AdmittedRelationalBranchBasis,
        target: RelationalSelectedCommit,
        selectors: &[Selector],
    ) -> Result<RelationalAuthoredWriteCursor, Denial> {
        self.admit_authored_write_history_cursor(
            execution,
            original,
            target,
            Selection::Ordinary(selectors),
        )
    }
    /// Complete conservative impact candidates for installed Source scopes.
    /// Shared-row scopes and disjoint fields on declared field kinds can appear.
    /// These pages
    /// prove neither exact query relevance nor correction cancellation.
    pub fn authored_managed_source_history_cursor(
        &self,
        execution: ExecutionRequest<'_, '_>,
        original: &AdmittedRelationalBranchBasis,
        target: RelationalSelectedCommit,
        selectors: &[RelationalAuthoredManagedSourceSelector],
    ) -> Result<RelationalAuthoredWriteCursor, Denial> {
        self.admit_authored_write_history_cursor(
            execution,
            original,
            target,
            Selection::Managed(selectors),
        )
    }
}
fn ordinary_bytes(context: &mut MapKernelContext<'_, '_>, key: &Selector) -> Result<u64, Denial> {
    let fields = match key {
        Selector::Record {
            scope: super::super::RelationalAuthoredWriteScope::Field(_, path),
            ..
        } => path.fields().len(),
        Selector::EntityFieldIndex { definition, .. } => {
            definition.locator.field_path().fields().len()
        }
        _ => 0,
    };
    super::super::admit_work(context, fields as u64)?;
    Ok(key.additional_charged_bytes())
}
impl RelationalRuntime {
    /// Select relevant authored writes strictly after the genuine source root.
    /// Prefix membership is taken from the target's canonical parent-derived
    /// generation, never from branch names or a supplied revision equality.
    /// Multi-parent histories and unavailable accelerators fail closed. Index
    /// predicates additionally require actual installed-definition coverage
    /// throughout the open source-to-target canonical interval.
    fn admit_authored_write_history_cursor(
        &self,
        execution: ExecutionRequest<'_, '_>,
        original: &AdmittedRelationalBranchBasis,
        target: RelationalSelectedCommit,
        input: Selection<'_>,
    ) -> Result<RelationalAuthoredWriteCursor, Denial> {
        let selectors = input.ordinary();
        if original.identity().runtime_instance_id() != self.runtime_instance_id()
            || target.runtime_instance_id() != self.runtime_instance_id()
        {
            return Err(Denial::ForeignOwner);
        }
        if target.observation().commit_id() != Some(target.commit_id()) {
            return Err(Denial::NotExactSelection);
        }
        if input.len() == 0 {
            return Err(Denial::EmptySelection);
        }
        super::super::run(execution, |context| {
            let managed_catalog = input.validate(self, context)?;
            for key in selectors {
                super::super::visit(context)?;
                if let Selector::Adjacency { anchor, .. } = key
                    && anchor.generation_value() == 0
                {
                    return Err(Denial::UnsupportedAdjacencySelector);
                }
                if let Selector::EntityFieldIndex { definition, .. } = key
                    && !super::super::RelationalAuthoredEntityIndexDefinition::supports_locator(
                        &definition.locator,
                    )
                {
                    return Err(Denial::UnsupportedIndexSelector);
                }
            }
            let original_commit = original
                .observation()
                .commit_id()
                .ok_or(Denial::NoSelectedCommit)?;
            if self
                .history_access()
                .immutable_commit_receipt(original_commit)
                .is_none()
            {
                return Err(Denial::SourceUnavailable);
            }
            let generation = self
                .history
                .authored_write_generation(target.commit_id())
                .ok_or(Denial::GenerationUnavailable)?;
            if generation.unsupported_merge {
                return Err(Denial::UnsupportedMergeInterval);
            }
            super::super::ordered_visit(context, generation.prefix.len())?;
            let original_ordinal = *generation
                .prefix
                .get(&original_commit)
                .ok_or(Denial::NotSelectedPrefix)?;
            let next = original_ordinal
                .checked_add(1)
                .ok_or(Denial::SizeOverflow)?;
            if let Some(catalog) = &managed_catalog {
                super::super::managed_source::require_interval(
                    &generation,
                    catalog,
                    next,
                    context,
                )?;
            }
            if selectors
                .iter()
                .any(|key| matches!(key, Selector::Adjacency { .. }))
            {
                super::super::ordered_visit(context, generation.incomplete_adjacency.len())?;
                if let Some((&ordinal, gap)) =
                    generation.incomplete_adjacency.first_at_or_after(&next)
                {
                    return Err(Denial::IncompleteAdjacencyHistory {
                        ordinal,
                        commit: gap.commit,
                        relation: gap.relation,
                        kind: gap.kind,
                        reason: gap.reason,
                    });
                }
            }
            super::super::admit_work(context, selectors.len() as u64)?;
            if selectors
                .iter()
                .any(|key| matches!(key, Selector::EntityFieldIndex { .. }))
            {
                super::super::ordered_visit(context, generation.incomplete_indexes.len())?;
                if generation
                    .incomplete_indexes
                    .first_at_or_after(&next)
                    .is_some()
                {
                    return Err(Denial::IncompleteIndexHistory);
                }
            }
            // Resolve at the real receiving catalog. A descriptive tuple is
            // insufficient even when older packets were complete for a smaller
            // installed roster. The first coverage and monotone-run boundary
            // prove every packet in this open selected interval.
            let catalog = self.indexes.definition_read_binding().snapshot();
            for key in selectors {
                super::super::visit(context)?;
                let Selector::EntityFieldIndex { definition, .. } = key else {
                    continue;
                };
                let introduction = catalog
                    .introduction(definition, context)?
                    .ok_or(Denial::IncompleteIndexHistory)?;
                if next > generation.ordinal {
                    continue;
                }
                if next < generation.index_run_start {
                    return Err(Denial::IncompleteIndexHistory);
                }
                super::super::ordered_visit(context, generation.index_coverage.len())?;
                let Some((_, coverage)) = generation.index_coverage.first_at_or_after(&next) else {
                    return Err(Denial::IncompleteIndexHistory);
                };
                if !coverage.extends(introduction) {
                    return Err(Denial::IncompleteIndexHistory);
                }
            }
            let memory = execution
                .reserve_retained_memory(
                    (std::mem::size_of::<RetainedAuthoredWriteSelection>()
                        + 2 * std::mem::size_of::<usize>()) as u64,
                )
                .map_err(Denial::Request)?;
            let bytes = selectors.iter().try_fold(0u64, |bytes, key| {
                super::super::visit(context)?;
                bytes
                    .checked_add(ordinary_bytes(context, key)?)
                    .ok_or(Denial::SizeOverflow)
            })?;
            let key_memory = execution
                .reserve_retained_memory(bytes)
                .map_err(Denial::Request)?;
            let mut keys = ExecutionRequestArrayBuilder::allocate(execution, input.len())?;
            for key in selectors {
                let bytes = ordinary_bytes(context, key)?;
                super::super::admit_work(context, bytes)?;
                keys.push(PostingKey::Ordinary(key.clone()))?;
            }
            if let Selection::Managed(selectors) = input {
                for selector in selectors {
                    super::super::visit(context)?;
                    keys.push(PostingKey::ManagedSource(ManagedSourceImpactKey {
                        program: selector.program,
                        scope: selector.scope,
                    }))?;
                }
            }
            // These sealed bases already own immutable root/history custody.
            // Sharing it must not readmit a historical commit's closed branch
            // as a live branch or acquire a second snapshot/retention identity.
            let original = original.clone();
            let target = target.observation().admitted_basis();
            Ok(RelationalAuthoredWriteCursor {
                source: Arc::new(RetainedAuthoredWriteSelection {
                    generation,
                    keys: keys.seal()?,
                    managed_catalog,
                    _key_memory: key_memory,
                    _original: original,
                    target,
                    _memory: memory,
                }),
                next,
            })
        })
    }
}
