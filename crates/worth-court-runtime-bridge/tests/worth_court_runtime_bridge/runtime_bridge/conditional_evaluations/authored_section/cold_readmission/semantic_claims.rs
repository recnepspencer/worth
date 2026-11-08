//! Fresh declared source bindings recover exact performed slots from authored claims.
use super::*;
use crate::conditional_evaluations::native_absence;
use std::sync::atomic::{AtomicUsize, Ordering};

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
    let medium = recorded_cut::RecordedCut::new(
        &[definition.clone()],
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
    let restored = owner.conditional_definitions(branch).next().unwrap();
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
            .authored_section(branch)
            .unwrap()
            .as_signal_section()
            .as_bytes(),
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
