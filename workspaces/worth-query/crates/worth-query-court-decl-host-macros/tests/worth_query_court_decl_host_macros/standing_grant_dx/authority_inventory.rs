//! W10A "explain authority": one inventory lists every installed ability and
//! capability a principal holds on a site, with the paths that allow it or
//! the policy cause that refuses it, behind the roster disclosure gate. Its
//! capability verdicts are admission's, so withdrawing an assignment moves
//! exactly the audit entry, as admission itself does.
use worth_query_host::facade::{
    eligibility::{
        WorthQueryAbilityEligibility, WorthQueryAbilityEligibilityDenial, WorthQueryAuthority,
        WorthQueryAuthorityInventory, WorthQueryAuthorityPath, WorthQueryAuthorityQuestion,
        WorthQueryAuthorityVerdict as Verdict, WorthQueryEligibilityDenial,
    },
    primary_graph::{
        WorthQueryApplicationAuthorizationExplanationCause as Cause,
        WorthQueryFixedRuntimeTimeSource,
    },
};

use super::grants::{AuditCapability, AuditOp, ViewAuditors};
use super::observers::{home, SiteRole};
use super::program::{caller, identity, QmsHome};
use super::schema::{Desktop, SetAssignmentStatus, SiteFacts};
use super::{at, temp_home_path};

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
    let clock = WorthQueryFixedRuntimeTimeSource::new(at(2026, 11, 15, 12, 0));
    let home = home(&path_on_disk, &clock);
    let alice = inventory(&home, "alice", "alice").expect("alice may see north's roster");
    assert_eq!(
        alice.subject_standing(),
        WorthQueryAbilityEligibility::Eligible
    );
    // One ability and five capabilities live on Site; every target is fixed.
    assert_eq!(alice.entries().len(), 6);
    assert!(alice
        .entries()
        .iter()
        .all(|entry| entry.verdict.is_allowed()));
    let view = WorthQueryAuthority::Ability {
        ability: "ViewAuditors".to_owned(),
        policy: "AuditorRosterPolicy".to_owned(),
    };
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
    let clock = WorthQueryFixedRuntimeTimeSource::new(at(2026, 11, 15, 12, 0));
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
