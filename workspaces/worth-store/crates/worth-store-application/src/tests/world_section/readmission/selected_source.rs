//! Original selected Home custody survives refusal, advance and foreign sources.
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
    let (builder, bridge, signal, _attachment) = super::recipe(home, request);
    let world = super::super::readmit_attached_world(home, builder, bridge, request).unwrap();
    let world_image = world
        .checkpoint_port()
        .unwrap()
        .prepare_home_checkpoint(request)
        .unwrap();
    let ordinary_native = home.relational.prepare_checkpoint(request).unwrap();
    let (members, member_funding) = ordinary_native.prepare_members(&[], request).unwrap();
    // Independent Home2 transport framing. The Native section is ACTUAL ordinary
    // producer output, not a fabricated Complete capture or a format-marker claim.
    let signal_image = signal.capture_authored_home_image(request).unwrap();
    let mut bytes = b"WAPHCKP1".to_vec();
    bytes.extend_from_slice(&3u32.to_le_bytes());
    bytes.extend_from_slice(&3u16.to_le_bytes());
    for (tag, body) in [
        (1u16, ordinary_native.bytes()),
        (4u16, world_image.bytes()),
        (
            worth_foundational::facade::DurableOwnerFamily::Signal.tag(),
            signal_image.as_bytes(),
        ),
    ] {
        bytes.extend_from_slice(&tag.to_le_bytes());
        bytes.extend_from_slice(&(body.len() as u64).to_le_bytes());
    }
    bytes.extend_from_slice(ordinary_native.bytes());
    bytes.extend_from_slice(world_image.bytes());
    bytes.extend_from_slice(signal_image.as_bytes());
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
    signal.prepare_home_stop(request).unwrap().commit();
    let report = world.lifecycle_port().close(request, &world).unwrap();
    assert!(report.retained_records().is_empty());
    drop((report, world, signal));
    (members, bytes)
}
fn home2_with_real_complete_native(
    home: &crate::ReadmittedApplicationHome,
    request: &ExecutionRequest<'_, '_>,
) -> (Vec<AnchoredRecordMemberLocator>, Vec<u8>) {
    let (builder, bridge, signal, _attachment) = super::recipe(home, request);
    let world = super::super::readmit_attached_world(home, builder, bridge, request).unwrap();
    let world_image = world
        .checkpoint_port()
        .unwrap()
        .prepare_home_checkpoint(request)
        .unwrap();
    // The Home image requires its actual original Native admission image and
    // the companion from this same World capture (D12 clean cutover).
    let (identity, through, digest) = world_image.home_admission_coordinates(request).unwrap();
    let captured = home
        .relational
        .complete_checkpoint_preparation()
        .prepare_home(identity, through, digest, request)
        .unwrap();
    let companion = world_image
        .capture_home_admission_companion(captured.admission_image(), request)
        .unwrap();
    let ordinary_native = captured
        .bind_home_admission_section(companion.bytes(), request)
        .unwrap();
    let (members, member_funding) = ordinary_native.prepare_members(&[], request).unwrap();
    // Both sections come from their real strong producers; framing alone
    // neither constructs Native Complete authority nor certifies selection.
    let signal_image = signal.capture_authored_home_image(request).unwrap();
    let mut bytes = b"WAPHCKP1".to_vec();
    bytes.extend_from_slice(&3u32.to_le_bytes());
    bytes.extend_from_slice(&3u16.to_le_bytes());
    for (tag, body) in [
        (1u16, ordinary_native.bytes()),
        (4u16, world_image.bytes()),
        (
            worth_foundational::facade::DurableOwnerFamily::Signal.tag(),
            signal_image.as_bytes(),
        ),
    ] {
        bytes.extend_from_slice(&tag.to_le_bytes());
        bytes.extend_from_slice(&(body.len() as u64).to_le_bytes());
    }
    bytes.extend_from_slice(ordinary_native.bytes());
    bytes.extend_from_slice(world_image.bytes());
    bytes.extend_from_slice(signal_image.as_bytes());
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
    signal.prepare_home_stop(request).unwrap().commit();
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
    let (members, bytes) = home2_with_real_complete_native(&home, &request);
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
    let crate::ApplicationHomeNativeSchemaPreparation::Existing { native, .. } =
        home.prepare_native_schema_for_home(&request).unwrap()
    else {
        panic!("existing source cannot initialize");
    };
    let runtime = native.runtime_instance_id();
    let expected = native
        .prepare_initial_schema_installation()
        .unwrap()
        .install(worth_relational::facade::schema::RelationalSchemaRegistry::new())
        .unwrap();
    let cancellation = CancellationSource::new();
    cancellation.cancel();
    let cancelled =
        SerialRequest::from_memory(SerialMemoryBudget::new(None), cancellation.token(), None);
    let refused = home
        .readmit(&expected, &ExecutionRequest::serial(&cancelled))
        .err()
        .expect("original request cancellation must refuse");
    assert!(matches!(
        refused.reason(),
        crate::ApplicationHomeOpenRefusal::Execution(_)
    ));
    advance_anchor(&serving, &members, bytes);
    let mut home = refused.into_home();
    assert_eq!(home.home_identity(), identity);
    let crate::ApplicationHomeNativeSchemaPreparation::Existing { native, .. } =
        home.prepare_native_schema_for_home(&request).unwrap()
    else {
        panic!("existing source cannot initialize");
    };
    assert_eq!(native.runtime_instance_id(), runtime);
    let mut home = home
        .readmit(&expected, &request)
        .unwrap()
        .consume(|home| home.share())
        .consume(|home| home);
    assert_eq!(home.home_identity(), identity);
    assert_eq!(
        home.context
            .selected_home_manifest(&request)
            .unwrap()
            .unwrap()
            .generation(),
        1
    );
    let competing = crate::recover_home(super::native_request(directory.path()), &request);
    assert!(
        matches!(competing, Err(crate::ApplicationHomeOpenRefusal::Physical(cause))
        if cause.kind() == crate::ApplicationHomePhysicalRefusalKind::MutationOwnerContended)
    );
    drop((home, serving));
}

#[test]
fn ordinary_selected_native_refuses_before_incarnation_and_native_or_world_publication() {
    let directory = tempfile::tempdir().unwrap();
    let serial = SerialRequest::from_memory(
        SerialMemoryBudget::new(None),
        CancellationSource::new().token(),
        None,
    );
    let request = ExecutionRequest::serial(&serial);
    let home = super::native_home(directory.path(), &request);
    invalid_home2_with_real_ordinary_native(&home, &request);
    drop(home.close(&request));
    let recovered = crate::recover_home(super::native_request(directory.path()), &request).unwrap();
    let (recovered, serving) = recovered.consume(|home| {
        let serving = home.serving.clone();
        (worth_proof::StoreRecovered::from_owner(home), serving)
    });
    let records = serving.anchored_records().unwrap();
    let anchors = [
        crate::registry::HOME_ANCHOR,
        crate::registry::CHECKPOINT_ANCHOR,
        crate::registry::RELATIONAL_INITIAL_INPUTS_ANCHOR,
    ];
    let before = anchors.map(|name| {
        records
            .read(AnchoredRecordName::new(name).unwrap(), 1 << 30)
            .unwrap()
            .unwrap()
    });
    let selected = crate::checkpoint::SelectedHomeSource::capture(&serving, &request).unwrap();
    let journals = crate::registry::HOME_JOURNALS.map(|journal| {
        worth_store::physical_runtime::OrderedJournalName::new(journal.name).unwrap()
    });
    let heads = journals.map(|name| selected.journal_head(&serving, name, &request).unwrap());
    assert!(matches!(
        crate::admit_home(recovered, &request),
        Err(crate::ApplicationHomeOpenRefusal::Relational(
            worth_store_relational::RelationalSectionReadmissionDenial::AdmissionImage(
                worth_relational::facade::durability::RelationalCheckpointAdmissionRefusal::PredecessorFormat {
                    protocol, version,
                }
            )
        )) if protocol == worth_foundational::facade::BoundaryProtocolIdentity::new(
            "worth.store.relational.checkpoint-section")
            && version.posture() == worth_foundational::facade::BoundaryProtocolUnsupportedVersionPosture::Retired
    ));
    for (name, original) in anchors.into_iter().zip(&before) {
        let actual = records
            .read(AnchoredRecordName::new(name).unwrap(), 1 << 30)
            .unwrap()
            .unwrap();
        assert_eq!(
            actual.version().generation(),
            original.version().generation()
        );
        assert_eq!(actual.bytes(), original.bytes());
    }
    let after = crate::checkpoint::SelectedHomeSource::capture(&serving, &request).unwrap();
    assert_eq!(
        journals.map(|name| after.journal_head(&serving, name, &request).unwrap()),
        heads
    );
    drop((selected, after, records, before));
    drop(serving);
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
