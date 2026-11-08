//! Current conditional definitions retain their exact performed claim subsets.
use super::super::{
    definition::DefinitionView,
    fields::Reader,
    layout::Layout,
    rows::Rows,
    targets::{DeclarationView, PerformedTargetView},
};
use super::{
    BridgeAuthoredInstallationBindings, BridgeAuthoredReadmissionDenial as D, PreparedDefinition,
};
use crate::correspondence::{
    AllocationSlotIdentity, BridgeSemanticClaimOrigin, BridgeSemanticCorrespondenceRegistration,
    InstalledCorrespondenceTarget,
};
use crate::facade::RuntimeBridge;
use std::{collections::BTreeMap, sync::Arc};
use worth_execution::ExecutionRequest;
use worth_foundational::facade::BridgeSemanticClaimReference;
use worth_signal::facade::durability::SignalAuthoredBranchInstallation;
use worth_signal::facade::{PartitionToken, SignalGraph};

pub(super) fn prepare(
    bridge: &RuntimeBridge,
    allocations: &crate::correspondence::SharedCorrespondenceAllocationRegistry,
    layout: &Layout<'_>,
    branch: &SignalAuthoredBranchInstallation<'_>,
    registrations: &BTreeMap<
        BridgeSemanticClaimReference,
        BridgeSemanticCorrespondenceRegistration,
    >,
    bindings: &mut dyn BridgeAuthoredInstallationBindings,
    backing: &Arc<super::backing::Backing>,
    execution: &ExecutionRequest<'_, '_>,
) -> Result<Vec<PreparedDefinition>, D> {
    let mut rows = Rows::new(layout.definitions)?;
    if rows.len() != branch.contracts().len() {
        return Err(D::ConditionalDefinitionMismatch);
    }
    backing.grow(
        execution,
        (std::mem::size_of::<PreparedDefinition>() as u64)
            .checked_mul(rows.len() as u64)
            .ok_or(D::section(
                super::super::BridgeAuthoredSectionDenial::SizeLimit,
            ))?,
    )?;
    let mut definitions = Vec::with_capacity(rows.len());
    while let Some(bytes) = rows.next()? {
        let mut input = Reader::new(bytes);
        let row = DefinitionView::read(&mut input)?;
        input.finish()?;
        let signal = branch
            .contracts()
            .iter()
            .find(|contract| contract.occurrence() == row.occurrence)
            .ok_or(D::ConditionalDefinitionMismatch)?;
        let source = row.readmit_source(execution, signal, Arc::clone(backing))?;
        let mut claims = Rows::new(row.claims)?;
        let mut correspondences = Vec::with_capacity(claims.len());
        let mut selected = Vec::with_capacity(claims.len());
        while let Some(bytes) = claims.next()? {
            let mut input = Reader::new(bytes);
            let reference = input.claim()?;
            let registration = registrations
                .get(&reference)
                .ok_or(D::ConditionalDefinitionMismatch)?;
            let targets = input.blob()?;
            let bytes = crate::correspondence::authored_performed_storage_bytes(
                Rows::new(targets)?.len(),
                targets.len() / super::super::fields::CLAIM_BYTES,
                targets.len(),
            )
            .ok_or(D::section(
                super::super::BridgeAuthoredSectionDenial::SizeLimit,
            ))?;
            backing.grow(execution, bytes)?;
            let (registration, targets) =
                performed(registration, targets, branch.graph(), backing, execution)?;
            input.finish()?;
            let allocation_guard = allocations
                .read()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let correspondence = crate::correspondence::readmit_performed_correspondence(
                bridge,
                &registration,
                branch.graph(),
                targets,
                &allocation_guard,
            )
            .map_err(D::Correspondence)?;
            drop(allocation_guard);
            selected.push(registration);
            correspondences.push(correspondence);
        }
        claims.finish()?;
        let providers =
            bindings.providers(layout.branch, &source.contract, &source.location, execution)?;
        let admission = crate::conditional_execution::provider_admission::admit_provider_set(
            Arc::clone(&source.contract),
            &providers,
        )
        .map_err(D::conditional)?;
        if !source.contract.is_valid()
            || !source.location.is_valid()
            || selected.len() != source.contract.dependency_count()
        {
            return Err(D::ConditionalDefinitionMismatch);
        }
        for ordinal in 0..source.contract.dependency_count() {
            if selected
                .iter()
                .filter(|registration| {
                    let candidate = registration.dependency();
                    candidate.dependency_ordinal() == ordinal
                        && candidate.source_node_identity() == source.contract.identity()
                        && candidate.source_stage_identity() == source.location.stage_identity()
                })
                .count()
                != 1
            {
                return Err(D::ConditionalDefinitionMismatch);
            }
        }
        let plan=crate::conditional_execution::semantic_observation_plan::compile_semantic_observation_plan(
            &source.contract,&selected).map_err(D::conditional)?;
        let mut dependencies = worth_signal::facade::AspectMask::EMPTY;
        let mut conditions = worth_signal::facade::AspectMask::EMPTY;
        for correspondence in &correspondences {
            for target in correspondence.authored_targets() {
                dependencies.insert(target.aspect);
                if source
                    .contract
                    .condition_dependency_ordinals()
                    .contains(&correspondence.dependency().dependency_ordinal())
                {
                    conditions.insert(target.aspect);
                }
            }
        }
        let definition = crate::conditional_execution::lowering::lower_signal_contract(
            &source.contract,
            dependencies,
            conditions,
        )
        .map_err(D::conditional)?;
        if &definition != signal.installed().authored_definition() {
            return Err(D::ConditionalDefinitionMismatch);
        }
        definitions.push(PreparedDefinition {
            source,
            signal: signal.installed().clone(),
            correspondences: Arc::from(correspondences),
            plan,
            providers,
            provider_admission: admission,
        });
    }
    rows.finish()?;
    Ok(definitions)
}
fn performed(
    registration: &BridgeSemanticCorrespondenceRegistration,
    bytes: &[u8],
    graph: &SignalGraph,
    backing: &super::backing::Backing,
    execution: &ExecutionRequest<'_, '_>,
) -> Result<
    (
        BridgeSemanticCorrespondenceRegistration,
        Vec<InstalledCorrespondenceTarget>,
    ),
    D,
> {
    let mut rows = Rows::new(bytes)?;
    let mut declarations = Vec::with_capacity(rows.len());
    let mut targets = Vec::with_capacity(rows.len());
    while let Some(bytes) = rows.next()? {
        let mut input = Reader::new(bytes);
        let row = PerformedTargetView::read(&mut input)?;
        input.finish()?;
        let declared = DeclarationView {
            node: row.node,
            partition: row.partition,
            mapping: row.mapping,
            slot: AllocationSlotIdentity::Exact(row.selected),
        };
        declarations.push(
            super::claims::declaration(graph, &declared, backing, execution)?
                .ok_or(D::ConditionalDefinitionMismatch)?,
        );
        let mut sources = Reader::new(row.sources);
        let mut allocation_sources =
            Vec::with_capacity(row.sources.len() / super::super::fields::CLAIM_BYTES);
        for _ in 0..row.sources.len() / super::super::fields::CLAIM_BYTES {
            allocation_sources.push(BridgeSemanticClaimOrigin::Recorded(sources.claim()?));
        }
        sources.finish()?;
        targets.push(InstalledCorrespondenceTarget {
            mapping_identity: Arc::from(row.mapping_basis),
            signal_graph_instance_id: graph.installed_graph_capability().graph_instance_id(),
            partition: PartitionToken(row.partition.to_owned()),
            node: super::claims::node(row.node),
            aspect: row.selected,
            precision: row.precision,
            admitted_source_widening: row.widening,
            allocation_sources,
        });
    }
    rows.finish()?;
    let mut selected = BridgeSemanticCorrespondenceRegistration::new(
        registration.dependency().clone(),
        declarations,
    )
    .map_err(|_| D::ConditionalDefinitionMismatch)?;
    selected.recorded_claim = registration.recorded_claim;
    Ok((selected, targets))
}
