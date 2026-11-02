//! Recorded correction checks the retained Source policy during preparation.
use super::super::super::correction::{self, note, recorded, state};
use super::*;
#[test]
fn source_only_read_revocation_refuses_recorded_correction_before_inverse_read() {
    lifecycle::run(|| {
        let setup = Setup::new();
        let request = setup
            .world
            .runtime()
            .request(&setup.principal, &setup.scope)
            .on_branch(setup.world.current_world());
        let operation = request
            .mutate(note::Input {
                parent: note::Parent::AuthorizedSource,
                directory: "catalog".into(),
                prefix: "sh".into(),
                expect_empty: false,
                replacement: "recorded authorized note".into(),
            })
            .without_source()
            .idempotency_owned(0x919c_0050u64)
            .unwrap_or_else(|(_, _, denial)| panic!("{denial:?}"))
            .retain_operation::<host::Program, correction::NoteAction>(&setup.world)
            .unwrap_or_else(|(_, denial)| panic!("{denial:?}"));
        let done = complete(operation.advance(&request), |next| next.advance(&request));
        let (_, evidence) = done
            .close_with_evidence(&request)
            .unwrap_or_else(|(_, denial)| panic!("{denial:?}"));
        let (WorthQuerySourceCompletion::Fresh { receipt, .. }
        | WorthQuerySourceCompletion::Recovered { receipt, .. }) = evidence;
        setup.set_readable(false, 0x919c_0051);
        let before = callbacks(&setup.calls.native);
        let head = request.retain_current_read().unwrap();
        let limits = || WorthQueryCorrectionHistoryLimits::new(None, std::num::NonZeroUsize::MIN);
        let selected = request.corrections().select(&receipt, limits()).unwrap();
        let denial = request
            .corrections()
            .reverse(&selected)
            .idempotency("2442397000")
            .limits(limits())
            .prepare()
            .err()
            .expect("Source-only READ revocation must refuse preparation before inverse handler");
        assert!(
            matches!(
                denial,
                WorthQueryApplicationCorrectionDenial::Derivation(
                    WorthQueryCorrectionDerivationDenial::SourceDiverged
                )
            ),
            "{denial:?}"
        );
        assert_eq!(
            callbacks(&setup.calls.native),
            before,
            "preparation performed no fresh Source/Parent query"
        );
        assert_eq!(
            request.retain_current_read().unwrap().selected_commit(),
            head.selected_commit()
        );
        setup.set_readable(true, 0x919c_0052);
        // Capture a new owning note after regrant. The old support still records
        // the uncancelled READ-policy history, so the lawful twin starts fresh.
        let fresh = request
            .mutate(note::Input {
                parent: note::Parent::AuthorizedSource,
                directory: "catalog".into(),
                prefix: "sh".into(),
                expect_empty: false,
                replacement: "fresh authorized note".into(),
            })
            .without_source()
            .idempotency_owned(0x919c_0053u64)
            .unwrap_or_else(|(_, _, denial)| panic!("{denial:?}"))
            .retain_operation::<host::Program, correction::NoteAction>(&setup.world)
            .unwrap_or_else(|(_, denial)| panic!("{denial:?}"));
        let done = complete(fresh.advance(&request), |next| next.advance(&request));
        let (_, fresh) = done
            .close_with_evidence(&request)
            .unwrap_or_else(|(_, denial)| panic!("{denial:?}"));
        let (WorthQuerySourceCompletion::Fresh { receipt, .. }
        | WorthQuerySourceCompletion::Recovered { receipt, .. }) = fresh;
        let selected = request.corrections().select(&receipt, limits()).unwrap();
        let prepared = request
            .corrections()
            .reverse(&selected)
            .idempotency("2442397001")
            .limits(limits())
            .prepare_operation::<recorded::Note, host::Program, correction::NoteAction>(
                &setup.world,
            )
            .unwrap();
        let advance = match prepared {
            WorthQueryOperationCorrectionPreparation::Prepared(prepared) => {
                prepared.publish().unwrap()
            }
            WorthQueryOperationCorrectionPreparation::Continued(advance) => advance,
            _ => panic!("the actual inverse retains its original source owner"),
        };
        complete(advance, |next| next.advance(&request))
            .close_with_evidence(&request)
            .unwrap_or_else(|(_, denial)| panic!("{denial:?}"));
        let actual = request
            .query(state::Read {
                directory: "catalog".into(),
            })
            .execute()
            .unwrap();
        assert_eq!(actual.rows()[0].note, "recorded authorized note");
    });
}
fn complete<O, C, T>(
    mut advance: WorthQuerySourceOnlyAdvance<O, C, T>,
    mut step: impl FnMut(O) -> WorthQuerySourceOnlyAdvance<O, C, T>,
) -> C {
    for _ in 0..16 {
        advance = match advance {
            WorthQuerySourceOnlyAdvance::Progress(next) => step(next),
            WorthQuerySourceOnlyAdvance::Completed(done) => return done,
            WorthQuerySourceOnlyAdvance::Refused { denial, .. } => panic!("{denial:?}"),
            WorthQuerySourceOnlyAdvance::Terminal(_) => panic!("unexpected terminal"),
        };
    }
    panic!("bounded owning operation must complete");
}
