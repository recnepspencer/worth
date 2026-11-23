//! An aligned read takes the committed index, where C granted Source READ.
//! The candidate must still grant it: a member of the same edit that revokes
//! the grant, even before the reader runs and without touching a row, leaves
//! the read unsupported and the edit unpublished.
use super::super::super::peer_scope::{AFTER_READ, STAGE_PROBE, STAGE_PROBE_PROTECTED};
use super::super::super::protected::status;
use super::*;
use worth_query_decl::facade::application_program::ApplicationNoOutputGraph;

/// One authoring stage whose decision probes the grant-governed Parent at
/// `catalog`, after `revoke` optionally withdraws the grant's READ, and
/// optionally followed by an unrelated third stage that must never settle an
/// already-revoked read (`B1-r3`).
fn probe_protected(
    revoke: bool,
    later_stage: bool,
    key: u64,
) -> (Vec<super::super::super::peer_scope::Observed>, bool, String) {
    let (world, _) = fresh_world();
    author(&world, "catalog", "c", key);
    let scope = authentication::request_scope();
    let principal = principal(&world, &scope);
    let request = world
        .runtime()
        .request(&principal, &scope)
        .on_branch(world.current_world());
    let mut envelope = request
        .edit_envelope()
        .idempotency(key + 1)
        .runtime_stages(&world)
        .ordered_authored()
        .unwrap();
    if revoke {
        envelope
            .push_stage::<ApplicationNoOutputGraph, _>(status::SetStatus {
                identity: "watch-access".into(),
                active: true,
                readable: false,
            })
            .unwrap();
    }
    // The stage writes under another Directory: `catalog` keeps its rows.
    envelope
        .push_stage::<ApplicationNoOutputGraph, _>(Edit::Item {
            directory: "A".into(),
            identity: "elsewhere".into(),
            name: "elsewhere".into(),
        })
        .unwrap();
    if later_stage {
        // An unrelated trailing authored stage; `AFTER_READ` disarms the probe
        // after Stage 1 so only Stage 1 carries the aligned protected read.
        envelope
            .push_stage::<ApplicationNoOutputGraph, _>(Edit::Directory {
                identity: "trailing".into(),
            })
            .unwrap();
        AFTER_READ.with(|hold| {
            *hold.borrow_mut() = Some(Box::new(|| {
                STAGE_PROBE.set(None);
                STAGE_PROBE_PROTECTED.set(false);
            }));
        });
    }
    OBSERVED.with(|all| all.borrow_mut().clear());
    STAGE_PROBE.set(Some("catalog"));
    STAGE_PROBE_PROTECTED.set(true);
    let outcome = envelope.execute_performed(ExecutionAllocationPolicy::SystemAllocation);
    STAGE_PROBE.set(None);
    STAGE_PROBE_PROTECTED.set(false);
    AFTER_READ.with(|hold| hold.borrow_mut().take());
    let (committed, detail) = match outcome {
        Ok(WorthQueryEditEnvelopeOutcome::Committed(_)) => (true, String::new()),
        Ok(WorthQueryEditEnvelopeOutcome::Uncommitted(cause)) => (false, format!("{cause:?}")),
        Ok(WorthQueryEditEnvelopeOutcome::StageExecutionDenied(cause)) => (
            false,
            format!("stage {}: {:?}", cause.binding(), cause.cause()),
        ),
        Ok(other) => (false, format!("{:?}", std::mem::discriminant(&other))),
        Err(cause) => (false, format!("{cause:?}")),
    };
    (
        OBSERVED.with(|all| std::mem::take(&mut *all.borrow_mut())),
        committed,
        detail,
    )
}

#[test]
fn an_aligned_read_refuses_when_an_earlier_stage_revoked_source_read() {
    lifecycle::run(|| {
        // With the grant intact the aligned protected read commits.
        let (observed, committed, detail) = probe_protected(false, false, 0x3105_0010);
        assert_eq!(observed, vec![Ok(vec!["c".to_owned()])], "{detail}");
        assert!(committed, "{detail}");
        // Revoked earlier in the same edit: the rows are unchanged, so the
        // read still comes from the committed index, but C's permission no
        // longer holds at the candidate and nothing publishes.
        let (observed, committed, detail) = probe_protected(true, false, 0x3105_0020);
        assert_eq!(observed, vec![Ok(vec!["c".to_owned()])], "{detail}");
        assert!(!committed, "the edit published on a revoked Source read");
        assert!(detail.contains("CoupledReadSupportChanged"), "{detail}");
        // Positive 3-stage control: intact grant with trailing stage commits.
        let (observed, committed, detail) = probe_protected(false, true, 0x3105_0028);
        assert_eq!(observed, vec![Ok(vec!["c".to_owned()])], "{detail}");
        assert!(committed, "{detail}");
        // Even with an unrelated trailing authored stage, staged settlement
        // must never attribute the pre-existing revocation to the later stage.
        let (observed, committed, detail) = probe_protected(true, true, 0x3105_0030);
        assert_eq!(observed, vec![Ok(vec!["c".to_owned()])], "{detail}");
        assert!(
            !committed,
            "trailing stage settled an already-revoked aligned Source read"
        );
        assert!(detail.contains("CoupledReadSupportChanged"), "{detail}");
    });
}
