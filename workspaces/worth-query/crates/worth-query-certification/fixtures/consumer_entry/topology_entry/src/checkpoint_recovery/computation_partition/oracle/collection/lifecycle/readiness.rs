//! Each lifecycle producer retains its exact singleton readiness dependency.
use super::*;
use crate::checkpoint_recovery::computation_partition::output_producer::{
    RegionOutputDomain, RegionOutputReadinessFamily, RegionOutputReadinessOperation,
    RegionOutputReadyNode,
};
use worth_query_host::facade::primary_graph::{
    WorthQueryConditionalApplicationRuntimeInstallation,
    WorthQueryConditionalRuntimeInstallationDenial,
};
use worth_query_host::facade::{application_contribution::*, domain};

pub(super) struct Readiness<const INITIAL: bool, const CONFLICT: bool = false, const ORPHAN: u8 = 0>;
impl<const INITIAL: bool, const CONFLICT: bool, const ORPHAN: u8>
    Readiness<INITIAL, CONFLICT, ORPHAN>
{
    fn conditional_binding(
        definition: &domain::WorthQueryDomainOperationDefinition<
            RegionOutputDomain,
            RegionOutputReadinessOperation,
            RegionOutputReadinessFamily,
        >,
    ) -> domain::WorthQueryApplicationConditionalOperationBinding<
        CheckpointSchema,
        TotalRegionOutput,
        RegionOutputInput,
        RegionOutputDomain,
        RegionOutputReadinessOperation,
        RegionOutputReadinessFamily,
    > {
        domain::WorthQueryApplicationConditionalOperationBinding::declare(
            TotalRegionOutput::reference::<CheckpointSchema>(),
            definition.reference(),
        )
    }
}
impl<const INITIAL: bool, const CONFLICT: bool, const ORPHAN: u8>
    WorthQueryApplicationConditionalBinding<CheckpointSchema>
    for Readiness<INITIAL, CONFLICT, ORPHAN>
{
    type Configuration = ();
    type Installed = ();
    type Operation = TotalRegionOutput;
    const IDENTITY: &'static str = if INITIAL {
        "checked-collection-initial-readiness-v1"
    } else {
        "checked-collection-preserve-readiness-v1"
    };
    const REQUIRED_PRODUCERS: &'static [&'static str] = if INITIAL {
        &[InitialProducer::IDENTITY]
    } else {
        &[PreserveProducer::IDENTITY]
    };
    fn package_contract(
        declaration: &worth_query_host::facade::declaration::application_schema::ErasedApplicationSchemaDeclaration,
    ) -> Result<
        WorthQueryApplicationConditionalPackageContract,
        worth_query_host::facade::application_contribution::WorthQueryOutputReadinessContractDenial,
    > {
        let definition = definition::<CONFLICT, ORPHAN>(declaration)?;
        let binding = Self::conditional_binding(&definition);
        Ok(WorthQueryApplicationConditionalPackageContract::new(
            definition.into_portable(),
            binding.portable().clone(),
            RegionOutputReadyNode::reference().node_identity(),
        ))
    }
    fn install(
        _: (),
        _: &WorthQueryApplicationConditionalProducerAccess<'_, CheckpointSchema>,
        installation: &mut WorthQueryConditionalApplicationRuntimeInstallation<CheckpointSchema>,
    ) -> Result<(), WorthQueryConditionalRuntimeInstallationDenial> {
        let definition = definition::<CONFLICT, ORPHAN>(
            installation.installed_schema().installed_declaration(),
        )?;
        let binding = Self::conditional_binding(&definition);

        let operation = installation
            .installed_schema()
            .installed_operation(TotalRegionOutput::reference::<CheckpointSchema>())
            .unwrap();
        let node = installation
            .installed_packages()
            .bind_conditional_application_operation(operation, &binding)
            .unwrap()
            .bind_node(RegionOutputReadyNode::reference())
            .unwrap();
        if INITIAL {
            installation.bind_output_readiness::<InitialProducer, _, _, _, _, _, _>(node, 0)
        } else {
            installation.bind_output_readiness::<PreserveProducer, _, _, _, _, _, _>(node, 0)
        }
    }
}

fn definition<const CONFLICT: bool, const ORPHAN: u8>(
    declaration: &worth_query_decl::facade::application_schema::ErasedApplicationSchemaDeclaration,
) -> Result<
    domain::WorthQueryDomainOperationDefinition<
        RegionOutputDomain,
        RegionOutputReadinessOperation,
        RegionOutputReadinessFamily,
    >,
    WorthQueryOutputReadinessContractDenial,
> {
    let identity = "collection-shared-readiness";
    let definition = WorthQueryOutputReadinessContractBuilder::for_field(
        declaration,
        crate::readiness::delivery_field(),
        domain::WorthQueryDomainOperationIdentity::new(identity, 1),
        "region-output-ready",
        crate::readiness::canonical_query(),
        domain::WorthQueryOperationProjectionRole::new("anchor").unwrap(),
        domain::WorthQueryExecutionStrategyName::new(identity).unwrap(),
        if CONFLICT {
            "collection-conflicting-readiness-v1"
        } else {
            identity
        },
    )?
    .field_dependency(
        declaration,
        crate::readiness::delivery_field(),
        WorthQueryReadinessFieldUse::DeliveryAndEligibility,
    )?
    .build_typed::<RegionOutputDomain, RegionOutputReadinessOperation, RegionOutputReadinessFamily>(
    )?
    .definition()
    .clone();
    if ORPHAN == 0 {
        return Ok(definition);
    }
    let mut semantics = definition.semantics().clone();
    if ORPHAN == 1 {
        semantics
            .conditional_nodes
            .push(additional_node(&semantics.conditional_nodes[0], false));
    } else {
        let stage = domain::WorthQueryPortableWorkflowStage::new(
            "orphan-stage",
            std::iter::empty::<&str>(),
            true,
            true,
            std::iter::empty(),
        )
        .with_semantics(domain::WorthQueryWorkflowStageSemantics {
            output: domain::WorthQueryWorkflowValueContract::Projection,
            graph_read_roles: vec!["primary".into()],
            terminal_result_states: semantics.terminal.result_states.clone(),
            failure_classes: semantics.terminal.failure_classes.clone(),
            resources: semantics.resources.clone(),
            // Same short name as the bound operation node. Only the full
            // location distinguishes this valid, unbound stage target.
            conditional_nodes: vec![additional_node(&semantics.conditional_nodes[0], true)],
            ..Default::default()
        });
        semantics.workflow = domain::WorthQueryOperationWorkflowContract::Declared(
            domain::WorthQueryPortableWorkflowDefinition::new("orphan-stage", [stage]),
        );
    }
    Ok(domain::WorthQueryDomainOperationDefinition::new(
        definition.identity().clone(),
        semantics,
    ))
}

fn additional_node(
    node: &domain::WorthQueryPortableConditionalNodeDeclaration,
    workflow_stage: bool,
) -> domain::WorthQueryPortableConditionalNodeDeclaration {
    let (identity, role, outputs, relationship) = if workflow_stage {
        (
            node.identity(),
            domain::WorthQueryConditionalNodeRole::WorkflowStage,
            vec![
                domain::WorthQueryConditionalNodeOutput::WorkflowStageOutput {
                    contract: domain::WorthQueryWorkflowValueContract::Projection,
                },
            ],
            domain::WorthQueryOutputRelationship::IsWorkflowStageOutput,
        )
    } else {
        (
            "unbound-output-ready",
            node.role(),
            node.outputs().to_vec(),
            node.output_relationship(),
        )
    };
    domain::WorthQueryPortableConditionalNodeDeclaration::declare(identity, role)
        .dependencies(node.dependencies().iter().cloned())
        .outputs(outputs)
        .required_context(node.required_context().iter().copied())
        .evaluation(node.condition().clone(), node.trigger().clone())
        .comparison(
            node.dependency_comparator().clone(),
            node.output_equivalence().clone(),
        )
        .artifact_policy(
            node.artifact_reuse_equivalence().clone(),
            node.maintenance(),
            node.artifact(),
        )
        .output_relationship(relationship)
        .finish()
        .expect("extra ordinary authored node is valid")
}
