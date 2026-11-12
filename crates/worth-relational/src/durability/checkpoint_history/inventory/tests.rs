//! The wire preflight funds the real shared roster before Serde grows it.
use super::*;
use worth_execution::{CancellationSource, SerialMemoryBudget, SerialRequest};
#[derive(serde::Serialize)]
struct Wire<T> {
    manifest: T,
}
#[derive(serde::Serialize)]
struct Description<T> {
    canonical_members: T,
}
fn rows() -> Vec<CanonicalCheckpointMember> {
    vec![
        CanonicalCheckpointMember {
            position: PatchStreamPosition(1),
            member: RelationalDurableMemberLocator {
                sha256: [3; 32],
                length: 123,
            },
        },
        CanonicalCheckpointMember {
            position: PatchStreamPosition(2),
            member: RelationalDurableMemberLocator {
                sha256: [4; 32],
                length: 127,
            },
        },
    ]
}
fn host(limit: u64) -> SerialRequest {
    SerialRequest::from_memory(
        SerialMemoryBudget::new(Some(limit)),
        CancellationSource::new().token(),
        None,
    )
}
fn bill(count: usize) -> u64 {
    let control = Layout::new::<(usize, usize)>()
        .extend(Layout::new::<Backing>())
        .unwrap()
        .0
        .pad_to_align()
        .size();
    (control
        + Layout::array::<CanonicalCheckpointMember>(count)
            .unwrap()
            .size()) as u64
}
#[test]
fn canonical_inventory_clone_keeps_only_original_roster_allocation() {
    let original = rows();
    let bytes = rmp_serde::to_vec(&original).unwrap();
    let wire = rmp_serde::to_vec_named(&Wire {
        manifest: Description {
            canonical_members: &original,
        },
    })
    .unwrap();
    let host = host(64 * 1024);
    let request = ExecutionRequest::serial(&host);
    let memory = prepare_decode(&wire, &request).unwrap();
    let mut actual: CanonicalCheckpointMembers = rmp_serde::from_slice(&bytes).unwrap();
    actual.attach(memory).unwrap();
    assert_eq!(actual.entries(), original);
    assert_eq!(
        host.memory().unwrap().charged_memory_bytes(),
        bill(original.len())
    );
    let clone = actual.clone();
    assert!(std::ptr::eq(
        clone.entries().as_ptr(),
        actual.entries().as_ptr()
    ));
    drop(actual);
    assert_eq!(
        host.memory().unwrap().charged_memory_bytes(),
        bill(original.len())
    );
    assert_eq!(
        clone.member(PatchStreamPosition(2)),
        Some(original[1].member)
    );
    drop(clone);
    assert_eq!(host.memory().unwrap().charged_memory_bytes(), 0);
}
#[test]
fn canonical_inventory_wire_requires_roster_before_decode_growth() {
    let original = rows();
    let wire = rmp_serde::to_vec_named(&Wire {
        manifest: Description {
            canonical_members: &original,
        },
    })
    .unwrap();
    let calibration = host(64 * 1024);
    let request = ExecutionRequest::serial(&calibration);
    let mut controller = None;
    request
        .run_native_with_checkpoint(|_| {
            controller = Some(calibration.memory().unwrap().charged_memory_bytes())
        })
        .unwrap();
    let controller = controller.unwrap();
    assert!(controller > 0);
    let target = host(controller + bill(original.len()) - 1);
    let request = ExecutionRequest::serial(&target);
    let refusal = prepare_decode(&wire, &request).unwrap_err();
    let (requested, admitted) = match refusal {
        crate::durability::data::RelationalRecoveryStop::Request(
            crate::execution::RelationalExecutionDenialCause::PolicyMemoryExhausted {
                requested,
                admitted,
                ancestor: 0,
            },
        ) => (requested, admitted),
        other => panic!("exact entered roster allocation required: {other:?}"),
    };
    assert_eq!(requested, bill(original.len()));
    assert_eq!(admitted, bill(original.len()) - 1);
    assert_eq!(target.memory().unwrap().charged_memory_bytes(), 0);
    let host = host(64 * 1024);
    let request = ExecutionRequest::serial(&host);
    let missing = rmp_serde::to_vec_named(&Wire {
        manifest: std::collections::BTreeMap::<String, u8>::new(),
    })
    .unwrap();
    assert!(prepare_decode(&missing, &request).is_err());
    let nonsequence = rmp_serde::to_vec_named(&Wire {
        manifest: Description {
            canonical_members: 7u8,
        },
    })
    .unwrap();
    assert!(prepare_decode(&nonsequence, &request).is_err());
    let empty = rmp_serde::to_vec_named(&Wire {
        manifest: Description {
            canonical_members: Vec::<CanonicalCheckpointMember>::new(),
        },
    })
    .unwrap();
    assert!(prepare_decode(&empty, &request).unwrap().is_none());
    assert_eq!(host.memory().unwrap().charged_memory_bytes(), 0);
}
