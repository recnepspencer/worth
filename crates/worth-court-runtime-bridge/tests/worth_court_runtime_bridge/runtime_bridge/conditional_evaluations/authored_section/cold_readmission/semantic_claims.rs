//! Fresh declared source bindings recover exact performed slots from authored claims.
use super::*;
use crate::conditional_evaluations::native_absence;
use std::sync::atomic::{AtomicUsize, Ordering};
mod fork_preparation;

struct Bindings {
    candidate: BridgeSemanticDependencyCandidate,
    contacts: Arc<native_absence::Contacts>,
    rebound: usize,
}
impl BridgeAuthoredInstallationBindings for Bindings {
    fn dependency(
        &mut self,
        branch: DurableSignalBranchIdentity,
        reference: BridgeSemanticClaimReference,
        description: &BridgeAuthoredDependencyDescription,
        _execution: &ExecutionRequest<'_, '_>,
    ) -> Result<BridgeSemanticDependencyCandidate, BridgeAuthoredReadmissionDenial> {
        assert_eq!(reference.definition().owner(), branch.owner());
        assert_eq!(
            description.contract(),
            &PortableAspectContract::from_contract(self.candidate.contract())
        );
        assert_eq!(description.projection_paths(), &[vec!["name".to_owned()]]);
        assert_eq!(description.node_identity(), "query:one");
        assert_eq!(description.dependency_ordinal(), 0);
        assert_eq!(description.graph_role(), "model");
        self.rebound += 1;
        Ok(self.candidate.clone())
    }
    fn providers(
        &mut self,
        _branch: DurableSignalBranchIdentity,
        contract: &BridgeConditionalContract,
        location: &BridgeConditionalLocation,
        _execution: &ExecutionRequest<'_, '_>,
    ) -> Result<BridgeConditionalProviderSet, BridgeAuthoredReadmissionDenial> {
        assert_eq!(contract.identity(), "query:one");
        assert_eq!(location.node_identity(), "query:one");
        Ok(native_absence::providers(Arc::clone(&self.contacts)))
    }
}
#[test]
fn cold_correspondence_rebinds_full_semantics_and_preserves_the_selected_slot_without_source_effects(
) {
    for replace_definition in [false, true] {
        let funding = request();
        let execution = ExecutionRequest::serial(&funding);
        let contacts = Arc::new(native_absence::Contacts::default());
        let readers = Arc::new(AtomicUsize::new(0));
        let (program, original) =
            native_absence::installed_builder(Arc::clone(&contacts), Arc::clone(&readers));
        let branch = branch();
        let node = original.signal_node();
        let occurrence = DurableSignalConditionalDefinitionOccurrence::from_recorded(
            DurableSignalNodeIdentity::from_recorded(branch, node.index(), node.generation()),
            DurableSignalConditionalOccurrence::from_recorded(branch.owner(), 1),
        )
        .unwrap();
        let definition = program
            .capture_authored_definition(
                &execution,
                SignalDefinitionTransitionIdentity::from_recorded(branch.owner(), 1),
                branch,
                &[occurrence],
            )
            .unwrap();
        let initial_definition = definition.clone();
        let definition = if replace_definition {
            let next = DurableSignalConditionalDefinitionOccurrence::from_recorded(
                occurrence.node(),
                DurableSignalConditionalOccurrence::from_recorded(branch.owner(), 2),
            )
            .unwrap();
            let section = BridgeAuthoredSection::capture_definition_successor(
                &execution,
                &definition,
                next,
                original.contract(),
                original.location(),
            )
            .unwrap();
            SignalDefinitionTransitionArtifact::capture(
                &execution,
                SignalDefinitionTransitionIdentity::from_recorded(branch.owner(), 2),
                branch,
                Some(definition.reference()),
                &definition.authored_image(),
                &definition.authored_runtime_image(),
                &[next],
                section.as_signal_section(),
            )
            .unwrap()
        } else {
            definition
        };
        let descriptor = SignalDurableBranchDescriptor::from_recorded(
            DurableSignalGeneration::from_recorded(
                DurableSignalCellIncarnation::from_recorded(branch, 1),
                0,
            ),
            None,
            DurableSignalAdmissionIdentity::from_recorded(branch.owner(), 1),
            definition.reference(),
        )
        .unwrap();
        let advance = SignalBranchAdvanceArtifact::capture(
            &execution,
            SignalBranchAdvanceIdentity::from_recorded(branch.owner(), 1),
            descriptor,
            None,
            SignalAuthoredBranchChange::Created {
                name: "semantic-branch".to_owned(),
                parent: None,
            },
        )
        .unwrap();
        let definitions = if replace_definition {
            vec![initial_definition, definition.clone()]
        } else {
            vec![definition.clone()]
        };
        let medium = recorded_cut::RecordedCut::new(
            &definitions,
            &[advance],
            &execution,
            "22222222-2222-4222-8222-222222222222",
        );
        let (fresh, fresh_lowering) =
            native_absence::installed_builder(Arc::clone(&contacts), Arc::clone(&readers));
        let candidate = fresh_lowering
            .semantic_dependencies()
            .next()
            .unwrap()
            .clone();
        assert_ne!(&candidate, original.semantic_dependencies().next().unwrap());
        let mut bindings = Bindings {
            candidate,
            contacts: Arc::clone(&contacts),
            rebound: 0,
        };
        let owner = fresh
            .readmit_authored_owner(
                medium,
                SignalRecordedIdentityContribution::empty(branch.owner()),
                &mut bindings,
                &execution,
            )
            .unwrap();
        assert_eq!(bindings.rebound, 1);
        let restored = owner
            .with_authored_branch(branch, |view| {
                view.conditional_definitions().next().unwrap()
            })
            .unwrap();
        assert_eq!(restored.signal_node(), node);
        assert_eq!(restored.correspondence_count(), 1);
        assert_eq!(restored.contract(), original.contract());
        assert_eq!(restored.location(), original.location());
        assert_eq!(
            restored.semantic_dependencies().next().unwrap(),
            &bindings.candidate
        );
        assert_eq!(
            owner
                .with_authored_branch(branch, |view| view
                    .authored_section()
                    .as_signal_section()
                    .as_bytes()
                    .to_vec())
                .unwrap(),
            definition.bridge_bytes()
        );
        let port = owner.signal_owner().source_port();
        let descriptor = port.observe(branch).unwrap();
        let handle = port.readmit_descriptor(&descriptor).unwrap();
        let (image, runtime) = port.capture_authored_images(&handle, &execution).unwrap();
        assert_eq!(image.as_bytes(), definition.authored_image().as_bytes());
        assert_eq!(
            runtime.as_bytes(),
            definition.authored_runtime_image().as_bytes()
        );
        assert_eq!(contacts.reads.load(Ordering::SeqCst), 0);
        assert_eq!(contacts.predicates.load(Ordering::SeqCst), 0);
        assert_eq!(contacts.computes.load(Ordering::SeqCst), 0);
        assert_eq!(readers.load(Ordering::SeqCst), 0);
    }
}
