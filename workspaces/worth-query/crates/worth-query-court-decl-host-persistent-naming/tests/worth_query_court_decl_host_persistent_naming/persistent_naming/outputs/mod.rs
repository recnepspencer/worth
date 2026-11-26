//! Slices 5C and 5D: installed operations name their own outputs. Births,
//! in-commit regeneration, splits, merges and retirements declared through
//! the candidate writer reach Relational's commit finalization, which mints
//! and validates them against the performed effects. Seams are read, split,
//! merged and retired by the names the decision reader holds, and a
//! later-role output may reference a name born by the same commit.
use super::*;
use probe::seams::SeamObservation;
use worth_query_decl::facade::application_naming::{ApplicationNameDiscriminator, NameSelection};
use worth_query_host::facade::application_naming::{
    LineageNameDenial, NameRef, NamedTarget, WorthQueryName, WorthQueryOutputBirth,
    WorthQueryOutputNameDenial, WorthQuerySameCommitName,
};

pub mod sculpt;
use sculpt::{Held, HeldSeam, Keeper, Plan, SeamKeeper, Step};

mod births;
mod ergonomics;
mod history;
mod invalidation;
mod lifecycle;
mod refusals;
mod relations;
mod same_commit;
mod seam_history;
mod seam_invalidation;
pub(crate) mod seam_names;

/// A member piece's side of a cut, optionally naming the cutting member.
/// Derived, so a recorded piece's discriminator also reads back.
#[derive(Clone, Debug, Eq, PartialEq, ApplicationNameDiscriminator)]
#[worth_query(identity = "worth.tests.persistent-naming.member-cut.v1")]
pub enum MemberCut {
    Side {
        side: u64,
        cutter: Option<WorthQueryName<Member>>,
    },
}

/// The same cut as [`MemberCut`], by a member born in the same commit. The
/// two share one identity and wire form, so a piece's name does not depend
/// on when its cutter was born.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FreshMemberCut {
    Side {
        side: u64,
        cutter: Option<WorthQuerySameCommitName<Member>>,
    },
}
worth_query_name_discriminator! {
    enum FreshMemberCut {
        identity: "worth.tests.persistent-naming.member-cut.v1",
        variants: { Side { side, cutter } }
    }
}

/// A piece cut along a seam linked in the same commit.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CutAlong {
    pub side: u64,
    pub seam: WorthQuerySameCommitName<Seam>,
}
worth_query_name_discriminator! {
    struct CutAlong {
        identity: "worth.tests.persistent-naming.cut-along.v1",
        fields: [side, seam]
    }
}

/// Which seam of a member authored in the same commit this is.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SeamOf {
    pub index: u64,
    pub member: WorthQuerySameCommitName<Member>,
}
worth_query_name_discriminator! {
    struct SeamOf {
        identity: "worth.tests.persistent-naming.seam-of.v1",
        fields: [index, member]
    }
}

/// Which of several parallel seams between one pair of members this is.
#[derive(Clone, Debug, Eq, PartialEq, ApplicationNameDiscriminator)]
#[worth_query(identity = "worth.tests.persistent-naming.seam-side.v1")]
pub struct SeamSide {
    pub index: u64,
}

/// Which side of a cut a piece lies on.
#[derive(Clone, Copy, Debug, Eq, PartialEq, ApplicationNameDiscriminator)]
#[worth_query(identity = "worth.tests.persistent-naming.cut-side.v1")]
pub enum CutSide {
    Positive,
    Negative,
}

/// A piece's side of a cut by another member, optionally along a seam.
/// Declared once: the cutter and the seam may each have been named by an
/// earlier commit or be named by this one.
#[derive(Clone, Debug, Eq, PartialEq, ApplicationNameDiscriminator)]
#[worth_query(identity = "worth.tests.persistent-naming.sided-cut.v1")]
pub enum SidedCut {
    By {
        cutter: NameRef<Member>,
        side: CutSide,
        along: Option<NameRef<Seam>>,
    },
}

pub type Outcome = WorthQueryApplicationMutationOutcome<edit::Denial, edit::Edited>;

pub fn birth(occurrence: &str, role: &str) -> WorthQueryOutputBirth {
    WorthQueryOutputBirth::new(occurrence, role)
}

pub fn piece(
    occurrence: &str,
    side: u64,
    cutter: Option<WorthQueryName<Member>>,
) -> WorthQueryOutputBirth {
    birth(occurrence, "piece").with_discriminator(&MemberCut::Side { side, cutter })
}

pub fn read(key: &str) -> Held {
    Held::Read(key.to_owned())
}

pub fn keys(keys: &[&str]) -> Vec<String> {
    keys.iter().map(|key| (*key).to_owned()).collect()
}

/// One live installed world and its operator's request.
pub struct World {
    pub host: world::Runtime,
    scope: worth_query_host::facade::admission::authenticated_principal::WorthQueryRequestScope,
    principal: WorthQueryAuthenticatedExternalPrincipal<Schema>,
}

impl World {
    pub fn install() -> Self {
        Self::over(world::Runtime::Memory(world::install()))
    }

    /// The seeded world on a new Directory Home at `location`.
    pub fn open_directory(location: std::path::PathBuf) -> Self {
        Self::over(world::Runtime::directory(location))
    }

    fn over(host: world::Runtime) -> Self {
        let scope = request_scope();
        let principal = authenticate_operator(host.installed_schema(), &scope);
        Self {
            host,
            scope,
            principal,
        }
    }

    /// Crashes the Directory Home and reopens it as a fresh installation;
    /// the operator authenticates anew against the reopened schema.
    pub fn crash_and_reopen(&mut self) {
        self.host.crash_and_reopen();
        self.scope = request_scope();
        self.principal = authenticate_operator(self.host.installed_schema(), &self.scope);
    }

    /// Captures a Directory Home checkpoint: a later reopen restores it
    /// and replays only the Journal tail after it.
    pub fn checkpoint(&self) {
        self.host.checkpoint();
    }

    pub fn request(&self) -> WorthQueryApplicationRequest<'_, '_, '_, Schema> {
        self.host.request(&self.principal, &self.scope)
    }

    /// Runs `plan` as one installed operation; `Err` carries a refusal the
    /// handler raised while authoring (before any effect).
    pub fn try_sculpt(&self, token: &str, plan: Plan) -> Result<Outcome, String> {
        sculpt::author(token, plan);
        self.request()
            .advanced()
            .mutate(sculpt::Intent {
                token: token.to_owned(),
            })
            .without_source()
            .idempotency(&format!("sculpt:{token}"))
            .execute_in_program(
                &self.host,
                worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
            )
            .map_err(|cause| format!("{cause:?}"))
    }

    /// Runs `plan` as one installed operation on the current world.
    pub fn sculpt(&self, token: &str, plan: Plan) -> Outcome {
        self.try_sculpt(token, plan)
            .unwrap_or_else(|cause| panic!("{token}: {cause}"))
    }

    pub fn committed(&self, token: &str, plan: Plan) {
        let outcome = self.sculpt(token, plan);
        assert!(
            matches!(outcome, Outcome::Committed { .. }),
            "{token}: {outcome:?}"
        );
    }

    /// Runs a probe naming `keys` and resolving every name held under
    /// `token`; its observations survive even when the probe cannot seal.
    pub fn probe(&self, token: &str, keys: &[&str], ordinal: u64) -> Vec<probe::Observation> {
        let _ = self
            .request()
            .advanced()
            .mutate(probe::Intent {
                token: token.to_owned(),
                keys: self::keys(keys),
                value: 200 + ordinal,
            })
            .without_source()
            .idempotency(&format!("probe:{token}:{ordinal}"))
            .execute_in_program(
                &self.host,
                worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
            );
        probe::take(token)
    }

    /// Runs a probe over `keys` and returns what it observed of seams,
    /// with the debug form of how the run ended (sealed or refused).
    pub fn probe_seams(
        &self,
        token: &str,
        keys: &[&str],
        ordinal: u64,
    ) -> (Vec<SeamObservation>, String) {
        let ended = self
            .request()
            .advanced()
            .mutate(probe::Intent {
                token: token.to_owned(),
                keys: self::keys(keys),
                value: 300 + ordinal,
            })
            .without_source()
            .idempotency(&format!("seam-probe:{token}:{ordinal}"))
            .execute_in_program(
                &self.host,
                worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
            );
        (probe::seams::take(token), format!("{ended:?}"))
    }

    /// Has model `a` hold `target` in its record from now on.
    pub fn hold(&self, token: &str, target: NamedTarget<Member>) {
        let outcome = self
            .request()
            .advanced()
            .mutate(stored::Intent {
                target,
                anchor: None,
            })
            .without_source()
            .idempotency(&format!("hold:{token}"))
            .execute_in_program(
                &self.host,
                worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
            );
        assert!(
            matches!(outcome, Ok(Outcome::Committed { .. })),
            "{token}: {outcome:?}"
        );
    }

    pub fn live(&self, key: &str) -> bool {
        world::member_identity(&self.host, self.host.current_world(), key).is_some()
    }
}

/// Relational's typed commit-time naming refusal behind `outcome`.
pub fn naming_refusal(outcome: &Outcome) -> Option<LineageNameDenial> {
    match outcome {
        Outcome::Commit(uncommitted) => uncommitted_refusal(uncommitted).cloned(),
        _ => None,
    }
}

/// The typed naming refusal an uncommitted request carries. A Directory
/// Home reports a Native refusal met while completing its durable candidate
/// as preparation pending, with that refusal as its Native preparation error.
pub fn uncommitted_refusal(
    uncommitted: &WorthQueryApplicationUncommitted,
) -> Option<&LineageNameDenial> {
    match uncommitted {
        WorthQueryApplicationUncommitted::Denied(denial) => denial.persistent_name_refusal(),
        WorthQueryApplicationUncommitted::ProductPreparationPending(pending) => pending
            .native_preparation_error()?
            .lineage_name_denial_ref(),
        _ => None,
    }
}

/// The typed refusal of `outcome`'s declared output names, lowered against
/// the commit basis before any effect was staged.
pub fn output_refusal(outcome: &Outcome) -> Option<&WorthQueryOutputNameDenial> {
    match outcome {
        Outcome::Commit(WorthQueryApplicationUncommitted::Denied(denial)) => {
            denial.persistent_name_output_denial()
        }
        _ => None,
    }
}

/// The name `relation_name_of` read for the seam labelled `label`.
pub fn seam_named(observed: &[SeamObservation], label: &str) -> WorthQueryName<Seam> {
    observed
        .iter()
        .find_map(|observation| match observation {
            SeamObservation::Named { label: l, name } if l == label => Some(*name),
            _ => None,
        })
        .unwrap_or_else(|| panic!("seam `{label}` must carry its name: {observed:?}"))
}

/// What a held seam name labelled `label` resolved to.
pub fn seam_resolution<'o>(observed: &'o [SeamObservation], label: &str) -> &'o SeamObservation {
    observed
        .iter()
        .find(|observation| match observation {
            SeamObservation::ResolvedTo { label: l, .. }
            | SeamObservation::Plural { label: l, .. }
            | SeamObservation::Retired { label: l }
            | SeamObservation::Ambiguous { label: l, .. }
            | SeamObservation::ResolveRefused { label: l, .. } => l == label,
            _ => false,
        })
        .unwrap_or_else(|| panic!("seam `{label}` must be resolved: {observed:?}"))
}

pub fn seam_between(label: &str, from: &str, to: &str) -> SeamObservation {
    SeamObservation::ResolvedTo {
        label: label.to_owned(),
        ends: (from.to_owned(), to.to_owned()),
    }
}

pub fn exact() -> NameSelection {
    NameSelection::Exact
}

pub fn named(observed: &[probe::Observation], key: &str) -> WorthQueryName<Member> {
    super::history::named(observed, key)
}

pub fn resolution<'o>(observed: &'o [probe::Observation], label: &str) -> &'o probe::Observation {
    super::history::resolution(observed, label)
}

pub fn resolved_to(label: &str, key: &str) -> probe::Observation {
    probe::Observation::ResolvedTo {
        label: label.to_owned(),
        key: key.to_owned(),
    }
}
