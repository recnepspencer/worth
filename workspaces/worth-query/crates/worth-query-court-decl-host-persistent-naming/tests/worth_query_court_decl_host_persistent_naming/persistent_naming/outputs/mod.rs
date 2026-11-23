//! Slice 5C: installed operations name their own outputs. Births, in-commit
//! regeneration, splits, merges and retirements declared through the
//! candidate writer reach Relational's commit finalization, which mints and
//! validates them against the performed effects.
use super::*;
use worth_query_host::facade::application_naming::{
    LineageNameDenial, WorthQueryName, WorthQueryOutputBirth, WorthQueryOutputNameDenial,
};

pub mod sculpt;
use sculpt::{Held, Keeper, Plan, Step};

mod history;
mod invalidation;
mod lifecycle;
mod refusals;
mod relations;

/// A member piece's side of a cut, optionally naming the cutting member.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MemberCut {
    Side {
        side: u64,
        cutter: Option<WorthQueryName<Member>>,
    },
}
worth_query_name_discriminator! {
    enum MemberCut {
        identity: "worth.tests.persistent-naming.member-cut.v1",
        variants: { Side { side, cutter } }
    }
}

/// Which of several parallel seams between one pair of members this is.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SeamSide {
    pub index: u64,
}
worth_query_name_discriminator! {
    struct SeamSide {
        identity: "worth.tests.persistent-naming.seam-side.v1",
        fields: [index]
    }
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
    pub host: world::Host,
    scope: worth_query_host::facade::admission::authenticated_principal::WorthQueryRequestScope,
    principal: WorthQueryAuthenticatedExternalPrincipal<Schema>,
}

impl World {
    pub fn install() -> Self {
        let host = world::install();
        let scope = request_scope();
        let principal = authenticate_operator(host.installed_schema(), &scope);
        Self {
            host,
            scope,
            principal,
        }
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

    pub fn live(&self, key: &str) -> bool {
        world::member_identity(&self.host, self.host.current_world(), key).is_some()
    }
}

/// Relational's typed commit-time naming refusal behind `outcome`.
pub fn naming_refusal(outcome: &Outcome) -> Option<LineageNameDenial> {
    match outcome {
        Outcome::Commit(WorthQueryApplicationUncommitted::Denied(denial)) => {
            denial.persistent_name_refusal().cloned()
        }
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
