//! Eligibility is the installed policy decision without the decision fact:
//! asking never commits, and every verdict agrees with what admission then
//! does, through allow and deny paths, scope, disabled and unmapped
//! identities, revocation and a pinned snapshot.
use worth_query_host::facade::eligibility::{
    WorthQueryAbilityEligibility as Verdict, WorthQueryAbilityEligibilityDenial,
    WorthQueryEligibilityDenial,
};
use worth_query_host::facade::{WorthQueryRefusalCategory, WorthQueryUserFacingRefusal};

use super::program::{identity, open};
use super::*;

/// Seeded verdicts for approving `D1`: alice holds approver; bob also
/// authors it (the deny path); erin holds nothing; dave's account is
/// disabled; the admin administers the site but holds no document role.
const D1_VERDICTS: [(&str, Verdict); 5] = [
    ("alice", Verdict::Eligible),
    ("bob", Verdict::Denied),
    ("erin", Verdict::Denied),
    ("dave", Verdict::Disabled),
    ("admin", Verdict::Denied),
];

#[test]
fn eligibility_agrees_with_admission_and_commits_nothing() {
    let path = temp_home_path("agreement");
    let clock = WorthQueryAdmissionClock::system();
    let home = open(&path, &clock);
    let (counter, _guard) = count_commits(&home);
    let before = home.committed_basis_version();

    let verdicts: Vec<_> = D1_VERDICTS
        .iter()
        .map(|(subject, _)| verdict(&session(&home, &clock, subject), "D1"))
        .collect();
    let expected: Vec<_> = D1_VERDICTS.iter().map(|(_, verdict)| *verdict).collect();
    assert_eq!(verdicts, expected);
    // Asking (twice over) minted nothing: no commit and no basis advance.
    for (subject, _) in D1_VERDICTS {
        let _ = verdict(&session(&home, &clock, subject), "D1");
    }
    assert_eq!(
        commits(&counter),
        0,
        "an eligibility question never commits"
    );
    assert_eq!(home.committed_basis_version(), before);

    // Admission then does exactly what each verdict said.
    for (subject, expected) in D1_VERDICTS {
        let acting = session(&home, &clock, subject);
        let outcome = acting.run(ApproveDocument {
            document: "D1".to_owned(),
            approved: true,
        });
        match expected {
            Verdict::Eligible => assert!(outcome.is_ok(), "{subject} was eligible"),
            _ => assert_eq!(
                outcome.expect_err("ineligible").refusal_category(),
                WorthQueryRefusalCategory::PermissionDenied,
                "{subject} was {expected:?}",
            ),
        }
    }
    assert_eq!(
        commits(&counter),
        1,
        "only alice's admitted approval commits"
    );
}

#[test]
fn scope_and_revocation_move_eligibility_and_admission_together() {
    let path = temp_home_path("revocation");
    let clock = WorthQueryAdmissionClock::system();
    let home = open(&path, &clock);
    let alice = session(&home, &clock, "alice");
    // Alice's approver assignment covers D1 only: D2 is outside its scope.
    assert_eq!(verdict(&alice, "D2"), Verdict::Denied);
    assert!(!approves(&alice, "D2"));
    // A pinned snapshot keeps the verdict of its commit across revocation.
    let pinned = alice.snapshot().expect("alice pins the current commit");
    revoke(&session(&home, &clock, "admin"), "alice-approves-d1");
    assert_eq!(
        pinned
            .eligibility(&approve("D1"))
            .expect("answered at the pin"),
        Verdict::Eligible
    );
    assert_eq!(verdict(&alice, "D1"), Verdict::Denied);
    assert!(!approves(&alice, "D1"), "revocation refuses the action too");
    drop(pinned);
    // A fresh grant restores both, through the same relationship path.
    assign(
        &session(&home, &clock, "admin"),
        "alice-again",
        2,
        "approver",
        "D2",
    );
    assert_eq!(verdict(&alice, "D2"), Verdict::Eligible);
    assert!(approves(&alice, "D2"));
}

#[test]
fn roster_lists_who_may_approve_only_behind_disclosure() {
    let path = temp_home_path("roster");
    let clock = WorthQueryAdmissionClock::system();
    let home = open(&path, &clock);
    let (counter, _guard) = count_commits(&home);
    let candidates: Vec<_> = ["alice", "bob", "erin", "dave", "ghost"]
        .into_iter()
        .map(identity)
        .collect();
    // The site admin holds no role on D1, so may not see who approves it.
    let refused = session(&home, &clock, "admin")
        .eligible_among(&approve("D1"), ViewApprovers::reference(), &candidates)
        .expect_err("the roster is not disclosed to the admin");
    assert!(matches!(
        refused,
        WorthQueryEligibilityDenial::Observation(
            WorthQueryAbilityEligibilityDenial::RosterNotDisclosed { .. }
        )
    ));
    // Bob authors D1, which discloses the roster (and only that).
    let roster = session(&home, &clock, "bob")
        .eligible_among(&approve("D1"), ViewApprovers::reference(), &candidates)
        .expect("an author sees the approver roster");
    let verdicts: Vec<_> = roster.iter().map(|(_, verdict)| verdict).collect();
    assert_eq!(
        verdicts,
        [
            Verdict::Eligible,
            Verdict::Denied,
            Verdict::Denied,
            Verdict::Disabled,
            Verdict::Unmapped,
        ]
    );
    let eligible: Vec<_> = roster.eligible().cloned().collect();
    assert_eq!(eligible, [identity("alice")]);
    assert_eq!(roster.verdict(&identity("ghost")), Some(Verdict::Unmapped));
    // Each roster verdict matches the candidate's own self-check.
    for (subject, expected) in [("alice", Verdict::Eligible), ("erin", Verdict::Denied)] {
        assert_eq!(verdict(&session(&home, &clock, subject), "D1"), expected);
    }
    assert_eq!(commits(&counter), 0, "rosters and self-checks never commit");
}

#[test]
fn a_pin_reports_its_commit_but_never_widens_who_may_ask() {
    let path = temp_home_path("pinned-asker");
    let clock = WorthQueryAdmissionClock::system();
    let home = open(&path, &clock);
    let alice = session(&home, &clock, "alice");
    let pinned = alice.snapshot().expect("alice pins while she approves D1");
    let admin = session(&home, &clock, "admin");
    revoke(&admin, "alice-approves-d1");
    // The pinned verdict is still the pinned commit's...
    assert_eq!(
        pinned
            .eligibility(&approve("D1"))
            .expect("answered at the pin"),
        Verdict::Eligible
    );
    // ...but the roster she may no longer see stays hidden at every pin.
    let hidden = pinned
        .eligible_among(
            &approve("D1"),
            ViewApprovers::reference(),
            &[identity("bob")],
        )
        .expect_err("disclosure is decided at the head");
    assert!(matches!(
        hidden,
        WorthQueryEligibilityDenial::Observation(
            WorthQueryAbilityEligibilityDenial::RosterNotDisclosed { .. }
        )
    ));
    // A disabled asker is not admitted to ask through an older pin either.
    admin
        .run(accounts::SetPrincipalStatus {
            directory: super::SITE.to_owned(),
            identity: identity("alice"),
            enabled: false,
        })
        .expect("the admin disables alice");
    assert!(matches!(
        pinned.eligibility(&approve("D1")),
        Err(WorthQueryEligibilityDenial::Observation(
            WorthQueryAbilityEligibilityDenial::Asker(_)
        ))
    ));
}

#[test]
fn a_refused_asker_cannot_tell_which_scopes_exist() {
    let path = temp_home_path("scope-oracle");
    let clock = WorthQueryAdmissionClock::system();
    let home = open(&path, &clock);
    // An unmapped or disabled asker is answered before any scope resolves.
    for (subject, expected) in [("frank", Verdict::Unmapped), ("dave", Verdict::Disabled)] {
        let asker = session(&home, &clock, subject);
        for document in ["D1", "D404"] {
            assert_eq!(asker.eligibility(&approve(document)).ok(), Some(expected));
            let roster = asker.eligible_among(&approve(document), ViewApprovers::reference(), &[]);
            assert!(
                matches!(
                    roster,
                    Err(WorthQueryEligibilityDenial::Observation(
                        WorthQueryAbilityEligibilityDenial::Asker(_)
                    ))
                ),
                "{subject} on {document}"
            );
        }
    }
    // An undisclosed roster asker cannot tell whether a document existed at
    // its pin: disclosure is decided at the head before the pin resolves.
    let carol = session(&home, &clock, "carol");
    let pinned = carol.snapshot().expect("carol pins before D6 exists");
    session(&home, &clock, "admin")
        .run(super::roles::PublishDocument {
            site: super::SITE.to_owned(),
            document: "D6".to_owned(),
        })
        .expect("the admin publishes D6 after the pin");
    for document in ["D1", "D6"] {
        let refused = pinned
            .eligible_among(
                &approve(document),
                ViewApprovers::reference(),
                &[identity("alice")],
            )
            .expect_err("carol holds no role on either document");
        assert!(
            matches!(
                refused,
                WorthQueryEligibilityDenial::Observation(
                    WorthQueryAbilityEligibilityDenial::RosterNotDisclosed { .. }
                )
            ),
            "{document}: {refused:?}"
        );
    }
}

#[test]
fn an_unknown_scope_refuses_the_question_without_a_verdict() {
    let path = temp_home_path("unknown-scope");
    let clock = WorthQueryAdmissionClock::system();
    let home = open(&path, &clock);
    let refused = session(&home, &clock, "alice")
        .eligibility(&approve("D404"))
        .expect_err("no such document");
    assert!(matches!(
        refused,
        WorthQueryEligibilityDenial::Observation(WorthQueryAbilityEligibilityDenial::Scope(_))
    ));
}
