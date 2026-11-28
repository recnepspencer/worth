//! Bind the native uniqueness delta to the same branch and applied draft as D.
use crate::branch::SelectedRelationalBranchRead;
use crate::runtime::RelationalPreparationRuntime;
use crate::transactions::data::TransactionCommitError;
use crate::validation::data::InvariantRule;
use worth_relational_indexes::internal_port::runtime::state::subsystems::indexing::definition_preparation::PreparedIndexCatalogOwnerPort as _;
use worth_relational_branch::internal_port::branch::structural_state::StructuralStateView;
use worth_relational_mutation::internal_port::validation::unique_field_probes::{
    self as probes, UniqueFieldProbeBatch, UniqueFieldScope,
};
use worth_relational_publication::internal_port::mvcc::publication::unique_field_admission;
use worth_relational_storage::internal_port::storage::overlay::access::OverlayStateView;

pub(super) fn prepare(
    runtime: &RelationalPreparationRuntime,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    input: &super::RelationalTransactionValidationInput,
    selected: &SelectedRelationalBranchRead<'_>,
    draft: &crate::authority::commit::phases::applied_mutation::AppliedMutationDraft,
    version: crate::identity::data::VersionId,
    plan: &crate::transactions::data::MergedCommitPlan,
) -> Result<UniqueFieldProbeBatch, TransactionCommitError> {
    let definitions = runtime.index_definitions.capture_catalog();
    let explicit = runtime
        .config
        .schema
        .invariant_catalog
        .registrations
        .iter()
        .filter_map(|registration| {
            let InvariantRule::UniqueEntityAspectField {
                entity_kind_id,
                backing_index_id,
                field_locator,
            } = &registration.rule
            else {
                return None;
            };
            (registration.failure_effect
                == crate::validation::data::InvariantFailureEffect::BlockCommit)
                .then_some((*entity_kind_id, *backing_index_id, field_locator))
        });
    let scopes = || {
        definitions
            .catalog
            .unique_entity_fields()
            .chain(explicit.clone())
    };
    let count = scopes().count();
    let mut batch = UniqueFieldProbeBatch::reserve(execution, count, definitions.next_index_id)?;
    let overlay = OverlayStateView::new(selected, draft.state());
    let before = StructuralStateView::new(selected, selected.version_id());
    let after = StructuralStateView::new(&overlay, version);
    for (entity_kind_id, backing_index_id, field_locator) in scopes() {
        let scope = UniqueFieldScope {
            entity_kind_id,
            backing_index_id,
            field_locator,
        };
        // A declaration repeated through the explicit catalog and its index
        // describes the same constraint and carries only one probe batch.
        if batch.claims().iter().any(|claim| {
            let previous = claim.scope();
            previous.backing_index_id == backing_index_id
                && previous.entity_kind_id == entity_kind_id
                && previous.field_locator == field_locator
        }) {
            continue;
        }
        if let Some(claims) = probes::collect(
            execution,
            scope,
            &before,
            &after,
            draft.state(),
            &plan.merged_intents,
            &[],
        )? {
            batch.push(claims);
        }
    }
    if let Some(target) = input.home_target()
        && let Some(prepared) = &target.index_definitions
        && let Some(coverage) = input.home_target_coverage()
    {
        // A Home transition rejudges the whole selected root for newly
        // introduced rules. There is no incumbent generation for those
        // rules. The prepared catalog and branch basis must both stay exact
        // until publication, which installs the verified target generation.
        for definition in prepared
            .captured_catalog()
            .introduced(definitions.next_index_id)
        {
            if let (
                Some(entity_kind_id),
                crate::indexes::data::DerivedIndexKind::EntityField { field_locator },
            ) = (definition.unique_entity_kind, &definition.kind)
            {
                let scope = UniqueFieldScope {
                    entity_kind_id,
                    backing_index_id: definition.index_id,
                    field_locator,
                };
                probes::collect(
                    execution,
                    scope,
                    &before,
                    &after,
                    draft.state(),
                    &[],
                    &coverage.records,
                )?;
            }
        }
    }
    unique_field_admission::admit(
        execution,
        &runtime.index_definitions,
        input.basis().identity().branch_id(),
        selected.selected().state().root(),
        batch.claims(),
    )
    .map_err(TransactionCommitError::from)?;
    Ok(batch)
}
