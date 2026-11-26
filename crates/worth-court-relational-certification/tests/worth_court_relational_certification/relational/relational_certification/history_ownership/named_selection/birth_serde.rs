//! Persisted births readmit only through verification (Phase 7, Arch 10).
//!
//! Checkpoint and Journal readmission decode every lineage event through
//! the verified birth adapter: parts that no longer re-encode to the stored
//! identity fail closed at decode. An identity does not bind its subject
//! kind, so a birth re-labelled into the other subject decodes, and the
//! derived name index refuses it (as do checkpoint event pages) rather than
//! file the name under another subject.
use super::*;
use serde_json::Value;
use worth_relational::facade::lineage::{
    LineageEventRecord, PersistentNameBranchIndex, PersistentNameCommittedBatch,
};

/// The persisted Create event of a genuinely named member, and its record.
fn persisted_birth() -> (Value, EntityId) {
    let host = serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let runtime = named_runtime();
    let member = owned_entity(execution, &runtime, &head(&runtime), "persisted", true);
    let commit = head(&runtime).observation().commit_id().unwrap();
    let envelope = runtime
        .replay()
        .canonical_commit_envelope(execution, commit)
        .unwrap()
        .expect("the named create's envelope");
    let event = envelope
        .lineage_events()
        .iter()
        .find(|event| event.name_birth().is_some())
        .expect("the persisted event carries its birth");
    (serde_json::to_value(event).unwrap(), member)
}

fn decode(value: Value) -> Result<LineageEventRecord, String> {
    serde_json::from_value(value).map_err(|refusal| refusal.to_string())
}

#[test]
fn a_genuine_persisted_birth_readmits_and_every_tampered_part_fails_closed() {
    let (genuine, _) = persisted_birth();
    let event = decode(genuine.clone()).expect("a genuine birth readmits");
    assert_eq!(serde_json::to_value(&event).unwrap(), genuine);
    let tampers: [(&str, fn(&mut Value)); 4] = [
        ("occurrence_key", |field| {
            *field = Value::from("cert.forged")
        }),
        ("role_path", |field| *field = Value::from("forged")),
        ("scheme_id", |field| {
            *field = Value::from("worth.cert.forged.v1")
        }),
        ("identity", |field| {
            field[0] = Value::from(field[0].as_u64().unwrap() ^ 1);
        }),
    ];
    for (field, tamper) in tampers {
        let mut tampered = genuine.clone();
        tamper(&mut tampered["name_birth"][field]);
        let refusal = decode(tampered).expect_err("a tampered birth never readmits");
        assert!(
            refusal.contains("corrupt persisted birth"),
            "`{field}`: {refusal}"
        );
    }
}

#[test]
fn a_birth_relabelled_into_another_subject_never_reaches_the_index() {
    let (genuine, member) = persisted_birth();
    let index = |event: LineageEventRecord| {
        let lineage = event.targets()[0];
        let batch = PersistentNameCommittedBatch::new(event.commit().commit_id)
            .with_lineage_events([event], [(lineage, member)], []);
        PersistentNameBranchIndex::new(BranchId("main".into()), None).apply(&batch)
    };
    index(decode(genuine.clone()).unwrap()).expect("the genuine birth indexes");
    let mut relabelled = genuine;
    relabelled["name_birth"]["subject_kind"] = Value::from(2);
    let refusal = index(decode(relabelled).expect("the identity does not bind a subject"))
        .expect_err("a relabelled birth never indexes");
    assert!(format!("{refusal:?}").contains("subject"), "{refusal:?}");
}
