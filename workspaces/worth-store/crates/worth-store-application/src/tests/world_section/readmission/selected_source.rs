//! Genuine C5 reads of administratively INVALID Home2, never Complete authority.
use super::*;
use worth_execution::{CancellationSource, SerialMemoryBudget, SerialRequest};
use worth_store::physical_runtime::{
    AnchoredRecordMember, AnchoredRecordMemberLocator, AnchoredRecordName,
    AnchoredRecordPublication, PhysicalMutationDeadline, ServingPhysicalRuntime,
};

fn invalid_home2_with_real_ordinary_native(
    home: &crate::ReadmittedApplicationHome,
    request: &ExecutionRequest<'_, '_>,
) -> (Vec<AnchoredRecordMemberLocator>, Vec<u8>) {
    let (builder, bridge, signal) = super::recipe(home);
    let medium = home.context.world_slot.as_ref().unwrap().medium();
    let world = builder
        .readmit_world_home(
            home.relational.runtime().owner_component_services(),
            bridge,
            medium,
            crate::world_section::checkpoint::readmission_source(home, request).unwrap(),
            request,
            home.relational.runtime(),
            home.relational.world_recovery_authority().unwrap(),
        )
        .unwrap();
    let world_image = world
        .checkpoint_port()
        .unwrap()
        .prepare_home_checkpoint(request)
        .unwrap();
    let ordinary_native = home.relational.prepare_checkpoint(request).unwrap();
    let (members, member_funding) = ordinary_native.prepare_members(&[], request).unwrap();
    // Independent Home2 transport framing. The Native section is ACTUAL ordinary
    // producer output, not a fabricated Complete capture or a format-marker claim.
    let mut bytes = b"WAPHCKP1".to_vec();
    bytes.extend_from_slice(&2u32.to_le_bytes());
    bytes.extend_from_slice(&2u16.to_le_bytes());
    for (tag, body) in [(1u16, ordinary_native.bytes()), (4u16, world_image.bytes())] {
        bytes.extend_from_slice(&tag.to_le_bytes());
        bytes.extend_from_slice(&(body.len() as u64).to_le_bytes());
    }
    bytes.extend_from_slice(ordinary_native.bytes());
    bytes.extend_from_slice(world_image.bytes());
    let published = home
        .context
        .serving
        .anchored_records()
        .unwrap()
        .publish(
            AnchoredRecordPublication::new(
                AnchoredRecordName::new(crate::registry::CHECKPOINT_ANCHOR).unwrap(),
                0,
                bytes.clone(),
                crate::open::physical_admission::placement(),
                PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
            )
            .with_members(members),
        )
        .unwrap();
    let members = published.members().to_vec();
    drop((ordinary_native, world_image, member_funding));
    let report = world.lifecycle_port().close(request, &world).unwrap();
    assert!(report.retained_records().is_empty());
    drop((report, world, signal));
    (members, bytes)
}
fn advance_anchor(
    serving: &ServingPhysicalRuntime,
    members: &[AnchoredRecordMemberLocator],
    mut bytes: Vec<u8>,
) {
    *bytes.last_mut().unwrap() ^= 1;
    serving
        .anchored_records()
        .unwrap()
        .publish(
            AnchoredRecordPublication::new(
                AnchoredRecordName::new(crate::registry::CHECKPOINT_ANCHOR).unwrap(),
                1,
                bytes,
                crate::open::physical_admission::placement(),
                PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
            )
            .with_members(
                members
                    .iter()
                    .map(|locator| AnchoredRecordMember::Carried { locator: *locator })
                    .collect(),
            ),
        )
        .unwrap();
}

#[test]
fn controlled_home_refusal_retains_original_selection_across_anchor_advance() {
    let directory = tempfile::tempdir().unwrap();
    let serial = SerialRequest::from_memory(
        SerialMemoryBudget::new(None),
        CancellationSource::new().token(),
        None,
    );
    let request = ExecutionRequest::serial(&serial);
    let home = super::native_home(directory.path(), &request);
    let (members, bytes) = invalid_home2_with_real_ordinary_native(&home, &request);
    // Ordinary owning disposal of the actual close outcome ends this fixture's
    // first ownership. This is NOT a physical Closed or bounded-Drop assertion.
    drop(home.close(&request));
    let admitted = crate::admit_home(
        crate::recover_home(super::native_request(directory.path()), &request).unwrap(),
        &request,
    )
    .unwrap();
    let (admitted, serving) = admitted.consume(|home| {
        let serving = home.context.serving.clone();
        (worth_proof::HomeAdmitted::from_owner(home), serving)
    });
    let mut home = crate::attach_relational(admitted, &request).unwrap();
    let identity = home.home_identity();
    let runtime = home.native_for_schema_installation().runtime_instance_id();
    let expected = home
        .native_for_schema_installation()
        .prepare_initial_schema_installation()
        .unwrap()
        .install(worth_relational::facade::schema::RelationalSchemaRegistry::new())
        .unwrap();
    let refused = home
        .readmit(&expected, &request)
        .err()
        .expect("ordinary Native cannot admit coupled Home2");
    assert!(matches!(
        refused.reason(),
        crate::ApplicationHomeOpenRefusal::Relational(
            worth_store_relational::RelationalSectionReadmissionDenial::CompleteNativeRequired
        )
    ));
    advance_anchor(&serving, &members, bytes);
    let mut home = refused.into_home();
    assert_eq!(home.home_identity(), identity);
    assert_eq!(
        home.native_for_schema_installation().runtime_instance_id(),
        runtime
    );
    let refused = home
        .readmit(&expected, &request)
        .err()
        .expect("same original selection remains unsupported");
    assert!(
        matches!(
            refused.reason(),
            crate::ApplicationHomeOpenRefusal::Relational(
                worth_store_relational::RelationalSectionReadmissionDenial::CompleteNativeRequired
            )
        ),
        "a second selector would encounter DifferentSelection at the installed Native source"
    );
    let competing = crate::recover_home(super::native_request(directory.path()), &request);
    assert!(
        matches!(competing, Err(crate::ApplicationHomeOpenRefusal::Physical(cause)) if cause.kind() == crate::ApplicationHomePhysicalRefusalKind::MutationOwnerContended)
    );
    drop(serving);
    // Retain the actual refusal through the lease oracle, then explicitly end
    // ownership by ordinary abandonment. No successful close is claimed here.
    drop(refused);
}

#[test]
fn selected_home_source_keeps_its_original_root_and_refuses_a_foreign_store() {
    let first = tempfile::tempdir().unwrap();
    let second = tempfile::tempdir().unwrap();
    let serial = SerialRequest::from_memory(
        SerialMemoryBudget::new(None),
        CancellationSource::new().token(),
        None,
    );
    let request = ExecutionRequest::serial(&serial);
    let home = super::native_home(first.path(), &request);
    let foreign = super::native_home(second.path(), &request);
    let (members, bytes) = invalid_home2_with_real_ordinary_native(&home, &request);
    let source =
        crate::checkpoint::SelectedHomeSource::capture(&home.context.serving, &request).unwrap();
    let original = source
        .selected(&home.context.serving, &request)
        .unwrap()
        .unwrap();
    assert_eq!(original.generation(), 1);
    assert!(matches!(
        source.selected(&foreign.context.serving, &request),
        Err(crate::ApplicationHomeCheckpointDenial::Read(
            worth_store::physical_runtime::AnchoredRecordReadFailure::ForeignReader
        ))
    ));
    advance_anchor(&home.context.serving, &members, bytes);
    assert_eq!(
        source
            .selected(&home.context.serving, &request)
            .unwrap()
            .unwrap()
            .generation(),
        1
    );
    let latest =
        crate::checkpoint::SelectedHomeSource::capture(&home.context.serving, &request).unwrap();
    assert_eq!(
        latest
            .selected(&home.context.serving, &request)
            .unwrap()
            .unwrap()
            .generation(),
        2
    );
    // Explicit ordinary abandonment only; source/read retirement and full D11
    // close semantics are not the assertion of this negative fixture.
    drop((source, latest, original, home, foreign));
}
