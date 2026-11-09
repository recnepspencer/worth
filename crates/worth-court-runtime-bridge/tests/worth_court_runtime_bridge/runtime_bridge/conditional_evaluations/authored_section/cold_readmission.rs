//! Readmission rebuilds every Bridge branch against actual cold Signal contracts.
use super::*;
use worth_signal::facade::durability::*;
mod home_stop;
mod recorded_cut;
mod semantic_claims;
mod shared_owner;

struct Bindings {
    providers: usize,
}
impl BridgeAuthoredInstallationBindings for Bindings {
    fn dependency(
        &mut self,
        _branch: DurableSignalBranchIdentity,
        _reference: BridgeSemanticClaimReference,
        _description: &BridgeAuthoredDependencyDescription,
        _execution: &ExecutionRequest<'_, '_>,
    ) -> Result<BridgeSemanticDependencyCandidate, BridgeAuthoredReadmissionDenial> {
        panic!("source-free definitions have no source dependencies")
    }
    fn providers(
        &mut self,
        _branch: DurableSignalBranchIdentity,
        contract: &BridgeConditionalContract,
        location: &BridgeConditionalLocation,
        _providers: &BridgeAuthoredProviderDescriptions,
        _execution: &ExecutionRequest<'_, '_>,
    ) -> Result<BridgeConditionalProviderSet, BridgeAuthoredReadmissionDenial> {
        assert_eq!(location.stage_identity(), Some("authored-stage"));
        assert_eq!(location.node_identity(), contract.identity());
        self.providers += 1;
        Ok(BridgeConditionalProviderSet::new()
            .authored_trigger(Requested)
            .authored_compute(VersionCompute))
    }
}
fn source(
    execution: &ExecutionRequest<'_, '_>,
) -> (
    Vec<SignalDefinitionTransitionArtifact>,
    Vec<SignalBranchAdvanceArtifact>,
) {
    let mut definitions = Vec::new();
    let mut advances = Vec::new();
    let mut parent = None;
    for ordinal in 1..=3 {
        let branch = DurableSignalBranchIdentity::from_recorded(branch().owner(), ordinal);
        let definition = super::capture_for(execution, branch, ordinal + 10);
        let cell = DurableSignalCellIncarnation::from_recorded(branch, ordinal);
        let descriptor = SignalDurableBranchDescriptor::from_recorded(
            DurableSignalGeneration::from_recorded(cell, 0),
            None,
            DurableSignalAdmissionIdentity::from_recorded(branch.owner(), ordinal),
            definition.reference(),
        )
        .unwrap();
        advances.push(
            SignalBranchAdvanceArtifact::capture(
                execution,
                SignalBranchAdvanceIdentity::from_recorded(branch.owner(), ordinal),
                descriptor.clone(),
                None,
                SignalAuthoredBranchChange::Created {
                    name: format!("authored-branch-{ordinal}"),
                    parent: parent.clone(),
                },
            )
            .unwrap(),
        );
        definitions.push(definition);
        parent = Some(descriptor);
    }
    (definitions, advances)
}
#[test]
fn cold_owner_rebuilds_conditional_definitions_and_retention_on_every_branch_for_either_build() {
    let funding = request();
    let execution = ExecutionRequest::serial(&funding);
    let (definitions, advances) = source(&execution);
    for build in [
        "11111111-1111-4111-8111-111111111111",
        "22222222-2222-4222-8222-222222222222",
    ] {
        let medium = recorded_cut::RecordedCut::new(&definitions, &advances, &execution, build);
        let mut bindings = Bindings { providers: 0 };
        let owner = Box::new(source_free_builder())
            .readmit_authored_owner(
                medium,
                SignalRecordedIdentityContribution::empty(branch().owner()),
                &mut bindings,
                &execution,
            )
            .unwrap();
        assert_eq!(bindings.providers, 6);
        let branches = owner.signal_owner().source_port().branches().unwrap();
        assert_eq!(branches.len(), 3);
        for ((identity, handle), definition) in branches.iter().zip(&definitions) {
            assert_eq!(
                owner
                    .with_authored_branch(*identity, |view| view
                        .authored_section()
                        .as_signal_section()
                        .as_bytes()
                        .to_vec())
                    .unwrap(),
                definition.bridge_bytes()
            );
            assert_eq!(
                owner
                    .with_authored_branch(*identity, |view| view
                        .with_conditional_definitions(|items| items.count()))
                    .unwrap(),
                2
            );
            let descriptor = owner
                .signal_owner()
                .source_port()
                .observe(*identity)
                .unwrap();
            assert_eq!(descriptor.definition(), definition.reference());
            assert_eq!(
                handle.parent_branch_id.map(|id| id.0),
                identity.ordinal().checked_sub(1).filter(|id| *id != 0)
            );
            let usage = owner
                .with_authored_branch(*identity, |view| view.conditional_retention())
                .unwrap();
            assert_eq!(usage.managed_clocks(), 0);
            assert_eq!(usage.reserved_temporal_intents(), 0);
            assert_eq!(usage.retained_definition_candidates(), 0);
            assert!(usage.retained_bytes() > 0);
            for lowering in owner
                .with_authored_branch(*identity, |view| {
                    view.with_conditional_definitions(|items| items.collect::<Vec<_>>())
                })
                .unwrap()
            {
                assert_eq!(lowering.location().stage_identity(), Some("authored-stage"));
                assert!(matches!(
                    lowering.contract().condition(),
                    BridgeConditionalCondition::OnDemand
                ));
                assert_eq!(lowering.correspondence_count(), 0);
                assert!(lowering.signal_basis_admission_identity().is_none());
                let source = owner.signal_owner().source_port();
                let branch = source.readmit_descriptor(&descriptor).unwrap();
                let world = owner.runtime_world_correspondence_port();
                let baseline = world.admit_signal_baseline(branch.basis()).unwrap();
                world.compare_current_exact(&baseline).unwrap();
                assert!(matches!(
                    world.admit_runtime_baseline(lowering.signal_graph_instance_id()),
                    Err(RuntimeWorldCorrespondenceAdmissionDenial::SignalBranchBasisRequired)
                ));
                let runtime = owner.conditional_execution(*identity).unwrap();
                let binding = runtime
                    .admit_conditional_signal_basis(&lowering, branch.basis())
                    .unwrap();
                let mut value = 41u64;
                let decision = runtime
                    .execute(
                        execution,
                        &binding,
                        BridgeConditionalExecutionRequest {
                            lowering: &lowering,
                            query_binding_identity: "cold-authored-query",
                            query_capability_identity: 1,
                            snapshot_identity: "cold-first-demand",
                            truth_branch_identity: None,
                            bridge_snapshot_identity: None,
                            execution_identity: "cold-execution",
                            attempt: 1,
                        },
                        &mut value,
                    )
                    .unwrap();
                assert!(decision.signal().computed_evaluation().is_some());
                assert_eq!(decision.signal().output_version(), 41);
                let (image, runtime_image) =
                    source.capture_authored_images(&branch, &execution).unwrap();
                assert_eq!(image.as_bytes(), definition.authored_image().as_bytes());
                assert_eq!(
                    runtime_image.as_bytes(),
                    definition.authored_runtime_image().as_bytes()
                );
            }
        }
    }
}
