//! W10A "explain authority": one inventory lists every installed ability and
//! capability a principal holds on a site, with the paths that allow it or
//! the policy cause that refuses it, behind the roster disclosure gate. Its
//! capability verdicts are admission's, so withdrawing an assignment moves
//! exactly the audit entry, as admission itself does.
use crate::time::controlled_at;
use worth_query_host::facade::runtime::time::ControlledTime;
use worth_query_host::facade::runtime::time::RawClockAdapter;
use worth_query_host::facade::runtime::{BootEpoch, ClockSourceIdentity, TimeUnavailable};
use worth_query_host::facade::{
    audit::{WorthQueryImpactDependency, WorthQueryImpactRole},
    eligibility::{
        WorthQueryAbilityEligibility, WorthQueryAbilityEligibilityDenial, WorthQueryAuthority,
        WorthQueryAuthorityInventory, WorthQueryAuthorityPath, WorthQueryAuthorityQuestion,
        WorthQueryAuthorityVerdict as Verdict, WorthQueryCapabilityEligibility,
        WorthQueryCapabilityEligibilityQuestion, WorthQueryEligibilityDenial,
    },
    local_calendar::WorthQueryTimeZone,
    primary_graph::WorthQueryApplicationAuthorizationExplanationCause as Cause,
};

use super::countersign::{CountersignCapability, CountersignOp, CountersignRequest};
use super::grants::{AuditCapability, AuditOp, ViewAuditors};
use super::observers::{home, SiteRole};
use super::program::{caller, identity, open, QmsHome};
use super::schema::{Desktop, Member, SetAssignmentStatus, SiteFacts};
use super::{at, november, temp_home_path};

fn inventory(
    home: &QmsHome,
    asker: &str,
    subject: &str,
) -> Result<WorthQueryAuthorityInventory, WorthQueryEligibilityDenial> {
    let question = WorthQueryAuthorityQuestion::new(
        Desktop,
        identity(subject),
        SiteFacts::site_id(),
        "north".to_owned(),
        ViewAuditors::reference(),
    );
    home.session(caller(home, asker).principal)
        .explain_principal_authority(&question)
}

/// alice's verdict countersigning `amount` on north (her limit is 100).
fn countersign_eligibility(home: &QmsHome, amount: u64) -> WorthQueryCapabilityEligibility {
    let request = CountersignRequest {
        site: "north".to_owned(),
        amount,
    };
    let question = WorthQueryCapabilityEligibilityQuestion::of_request(
        Desktop,
        CountersignCapability::reference(),
        CountersignOp::reference(),
        &request,
    )
    .expect("the countersign request projects");
    home.session(caller(home, "alice").principal)
        .eligible_for_capability(&question)
        .unwrap_or_else(|denial| panic!("the countersign question is answered: {denial:?}"))
}

fn capability(name: &str, operation: &str) -> WorthQueryAuthority {
    WorthQueryAuthority::Capability {
        capability: name.to_owned(),
        operation: operation.to_owned(),
    }
}

fn path(relations: &[&str]) -> WorthQueryAuthorityPath {
    WorthQueryAuthorityPath {
        relations: relations
            .iter()
            .map(|relation| (*relation).to_owned())
            .collect(),
    }
}

#[test]
fn an_inventory_explains_every_installed_authority_on_a_scope() {
    let path_on_disk = temp_home_path("authority-inventory");
    let clock = controlled_at(at(2026, 11, 15, 12, 0));
    let home = home(&path_on_disk, &clock);
    let alice = inventory(&home, "alice", "alice").expect("alice may see north's roster");
    assert_eq!(
        alice.subject_standing(),
        WorthQueryAbilityEligibility::Eligible
    );
    // One ability and seven capabilities live on Site. Every fixed target is
    // allowed; a countersignature names its amount and evidence its purpose,
    // so only a request asks them.
    assert_eq!(alice.entries().len(), 8);
    assert_eq!(alice.allowed().count(), 6);
    let countersign = capability("CountersignCapability", "CountersignOp");
    assert_eq!(alice.verdict(&countersign), Some(&Verdict::RequiresRequest));
    let evidence = capability("EvidenceCapability", "InspectEvidenceOp");
    assert_eq!(alice.verdict(&evidence), Some(&Verdict::RequiresRequest));
    assert_eq!(
        countersign_eligibility(&home, 100),
        WorthQueryCapabilityEligibility::Eligible
    );
    assert_eq!(
        countersign_eligibility(&home, 101),
        WorthQueryCapabilityEligibility::Denied(Cause::MissingCapability)
    );
    let view = WorthQueryAuthority::Ability {
        ability: "ViewAuditors".to_owned(),
        policy: "AuditorRosterPolicy".to_owned(),
    };
    // Both ViewAuditors rules (any grant, a review grant) hold over the one
    // holder → site chain, which is listed once.
    let grant_path = Verdict::Allowed {
        through: vec![path(&["RoleHolder", "GrantSite"])],
    };
    assert_eq!(alice.verdict(&view), Some(&grant_path));
    let audit = capability("AuditCapability", "AuditOp");
    assert_eq!(alice.verdict(&audit), Some(&grant_path));

    // bob holds no grant: every capability names its policy cause.
    let bob = inventory(&home, "alice", "bob").expect("alice may ask about bob");
    assert_eq!(bob.verdict(&view), Some(&Verdict::Denied(None)));
    assert_eq!(
        bob.verdict(&audit),
        Some(&Verdict::Denied(Some(Cause::MissingCapability)))
    );
    assert_eq!(bob.allowed().count(), 0);

    // Withdrawing alice's assignment moves exactly the audit entry, and
    // admission refuses it for the same cause.
    home.session(caller(&home, "alice").principal)
        .run(SetAssignmentStatus {
            assignment: "alice-north".to_owned(),
            status: "disabled".to_owned(),
        })
        .unwrap_or_else(|denial| panic!("assignment status commits: {denial:?}"));
    let withdrawn = inventory(&home, "alice", "alice").expect("still disclosed");
    let role = SiteRole {
        capability: AuditCapability::reference(),
        operation: AuditOp::reference(),
    };
    let cause = role.agreed(&home, "alice", "north");
    assert!(cause.is_some());
    assert_eq!(withdrawn.verdict(&audit), Some(&Verdict::Denied(cause)));
    assert_eq!(withdrawn.allowed().count(), 5);
}

#[test]
fn an_inventory_is_disclosed_only_to_a_holder_of_the_disclosure_ability() {
    let path_on_disk = temp_home_path("authority-disclosure");
    let clock = controlled_at(at(2026, 11, 15, 12, 0));
    let home = home(&path_on_disk, &clock);
    let refused = inventory(&home, "bob", "alice").expect_err("bob holds no role over north");
    assert!(
        matches!(
            refused,
            WorthQueryEligibilityDenial::Observation(
                WorthQueryAbilityEligibilityDenial::RosterNotDisclosed { .. }
            )
        ),
        "{refused:?}"
    );
    let carol = inventory(&home, "alice", "carol").expect("an unmapped subject is answered");
    assert_eq!(
        carol.subject_standing(),
        WorthQueryAbilityEligibility::Unmapped
    );
    assert!(carol.entries().is_empty());
}

#[test]
fn two_policy_rules_over_one_chain_name_each_impact_role_once() {
    let path_on_disk = temp_home_path("authority-impact");
    let clock = controlled_at(at(2026, 11, 15, 12, 0));
    let home = home(&path_on_disk, &clock);
    let impact = home
        .session(caller(&home, "alice").principal)
        .entity_impact(Member::reference());
    // Each rule makes Member the principal and steps along RoleHolder; the
    // roles interleave, so only a non-adjacent deduplication lists each once.
    let roles: Vec<String> = impact
        .dependencies()
        .iter()
        .filter_map(|dependency| match dependency {
            WorthQueryImpactDependency::AbilityPolicy { ability, role, .. }
                if ability == "ViewAuditors" =>
            {
                Some(match role {
                    WorthQueryImpactRole::PrincipalEntity => "principal".to_owned(),
                    WorthQueryImpactRole::Traversal { edge, .. } => edge.relation().to_owned(),
                    other => format!("{other:?}"),
                })
            }
            _ => None,
        })
        .collect();
    assert_eq!(roles, ["principal", "RoleHolder"]);
}

/// The fixed trusted clock, counting every read of the current instant.
#[derive(Clone)]
struct CountingClock {
    inner: std::sync::Arc<ControlledTime>,
    reads: std::sync::Arc<std::sync::atomic::AtomicUsize>,
}

impl RawClockAdapter for CountingClock {
    fn read_monotonic_ticks(&self) -> (u64, BootEpoch) {
        self.inner.read_monotonic_ticks()
    }

    fn source_identity(&self) -> ClockSourceIdentity {
        self.inner.source_identity()
    }

    fn read_wall_unix_nanos(&self) -> Result<u64, TimeUnavailable> {
        self.reads.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        self.inner.read_wall_unix_nanos()
    }
}

#[test]
fn one_inventory_reads_the_trusted_clock_once_for_every_capability() {
    let clock = std::sync::Arc::new(CountingClock {
        inner: controlled_at(at(2026, 11, 15, 12, 0)),
        reads: Default::default(),
    });
    let denver = WorthQueryTimeZone::from_tz_variable("MST7MDT,M3.2.0,M11.1.0").unwrap();
    let home = open(
        &temp_home_path("authority-one-instant"),
        clock.clone(),
        Some(denver),
        november(),
    )
    .unwrap_or_else(|denial| panic!("QMS Home opens: {denial}"));
    let reads = || clock.reads.load(std::sync::atomic::Ordering::SeqCst);
    for _ in 0..2 {
        let before = reads();
        let alice = inventory(&home, "alice", "alice").expect("alice may see north's roster");
        // Five fixed-target capabilities are allowed; one reading judged them.
        assert_eq!(alice.allowed().count(), 6);
        assert_eq!(reads() - before, 1);
    }
}
