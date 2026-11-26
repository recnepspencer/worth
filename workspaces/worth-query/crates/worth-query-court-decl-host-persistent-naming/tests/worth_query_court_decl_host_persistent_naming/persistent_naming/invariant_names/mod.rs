//! #418: an installed invariant reads persistent names over the commit under
//! evaluation. `NameWatch` runs at the commit boundary of its own program
//! (`WatchWorld`); the shared persistent-naming program never installs it.
//! For each touched member it reads, in both scope planning and evaluation,
//! the proposed name and recorded birth, the committed name of the same
//! record, and whether the committed view sees the proposed name's birth. It
//! logs what it saw into its world's own log, and refuses a commit whose
//! proposed birth decodes a forbidden cut side.
use super::outputs::sculpt::{Plan, Step};
use super::outputs::MemberCut;
use super::*;
use std::cell::RefCell;
use std::sync::{Arc, Mutex};
use worth_query_decl::facade::application_naming::{RecordedNameBirth, WorthQueryName};
use worth_query_host::facade::application_invariants::{
    WorthQueryApplicationInvariantContext, WorthQueryApplicationInvariantEntity,
    WorthQueryApplicationInvariantEntityBinding, WorthQueryApplicationInvariantError,
    WorthQueryApplicationInvariantFieldBinding, WorthQueryApplicationInvariantReadView,
    WorthQueryApplicationInvariantRule, WorthQueryApplicationInvariantSchemaResolver,
    WorthQueryApplicationInvariantScopePlanner, WorthQueryApplicationInvariantVerdict,
    WorthQueryInvariantNameDenial,
};

mod continuity;
mod courts;
mod history;
pub(crate) mod refusals;
mod world;
pub use world::WatchWorld;

pub struct NameWatch;
impl ApplicationInvariantMarkerIdentity<Schema> for NameWatch {
    const IDENTIFIER: &'static str = "persistent-naming-name-watch";
    const MAJOR: u16 = 1;
    const MINOR: u16 = 0;
}

/// Declares `NameWatch` only while the watched world installs: every other
/// program's rule list leaves it out.
pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    if installing().is_none() {
        return schema;
    }
    let targets = || {
        [
            ApplicationInvariantScopeTarget::Entity("Member".into()),
            ApplicationInvariantScopeTarget::Entity("Model".into()),
        ]
    };
    schema.invariant(ApplicationInvariantDefinition::new(
        NameWatch::reference(),
        ApplicationInvariantExecutionPoint::CommitBoundary,
        RelationalInvariantWorkBudget::new(None),
        ApplicationInvariantOperationalContract::new(
            ApplicationInvariantEnforcement::BlockCommit,
            [ApplicationInvariantGroup::SchemaCompliance],
            targets(),
            targets(),
            "primary-relational-provider",
            ApplicationInvariantCostPosture::Touched,
        ),
    ))
}

#[derive(Clone, Debug, PartialEq, Eq, WorthValue)]
#[worth_value(crate = "worth_query_decl::facade::application_schema")]
pub enum Refusal {
    ForbiddenSide { key: String, side: u64 },
}
worth_query_value_binding!(pub RefusalBinding for Refusal { identity: "persistent-naming.name-watch-refusal.v1", scalar: String, canonical: ValueDecodeBounds::new(256, 64, 8, 8), });

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Phase {
    Planned,
    Evaluated,
}

pub type Named = Result<WorthQueryName<Member>, WorthQueryInvariantNameDenial>;
pub type Born = Result<RecordedNameBirth<Member>, WorthQueryInvariantNameDenial>;

/// What `NameWatch` saw of one record in one phase.
#[derive(Clone, Debug)]
pub enum Seen {
    Member {
        phase: Phase,
        key: String,
        /// `proposed().name_of`.
        proposed: Named,
        /// `proposed().birth_of` of that name.
        birth: Option<Born>,
        /// `committed().name_of` of the same record; `None` when the record
        /// is not in the committed view.
        committed: Option<Named>,
        /// `committed().birth_of` of the proposed name.
        committed_birth: Option<Born>,
    },
    /// A member the commit deletes: read in the committed view only.
    Gone {
        key: String,
        committed: Named,
        /// Whether the proposed view still observes the record.
        reobserved: bool,
        /// `proposed().name_of` handed the committed view's handle.
        cross_view: String,
    },
    /// A model: its kind declares no name scheme.
    Model { key: String, name: String },
}

/// One world's watch: its log and the cut side it refuses.
#[derive(Default)]
pub struct Watch {
    pub seen: Vec<Seen>,
    pub forbidden_side: Option<u64>,
}
pub type Shared = Arc<Mutex<Watch>>;

thread_local! {
    static INSTALLING: RefCell<Option<Shared>> = const { RefCell::new(None) };
}

/// The watch the installation running on this thread registers, if any.
pub fn installing() -> Option<Shared> {
    INSTALLING.with(|slot| slot.borrow().clone())
}

pub fn install(
    setup: &mut WorthQueryApplicationContributionSetup<'_, Schema>,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    let Some(watch) = installing() else {
        return Ok(());
    };
    setup.invariant(
        NameWatch::reference(),
        ApplicationInvariantExecutionPoint::CommitBoundary,
        move |resolver| resolve(resolver, watch),
    )
}

type Error = WorthQueryApplicationInvariantError<Refusal>;
type Handle<E> = WorthQueryApplicationInvariantEntity<Schema, E>;

pub struct Rule {
    watch: Shared,
    member: WorthQueryApplicationInvariantEntityBinding<Schema, Member>,
    model: WorthQueryApplicationInvariantEntityBinding<Schema, Model>,
    member_key: WorthQueryApplicationInvariantFieldBinding<Schema, Member, String>,
    model_key: WorthQueryApplicationInvariantFieldBinding<Schema, Model, String>,
}

fn resolve(
    resolver: &WorthQueryApplicationInvariantSchemaResolver<'_, Schema>,
    watch: Shared,
) -> Result<Rule, String> {
    Ok(Rule {
        watch,
        member: resolver.typed_entity(Member::reference()).ok_or("member")?,
        model: resolver.typed_entity(Model::reference()).ok_or("model")?,
        member_key: resolver
            .typed_field(MemberKey::at("MemberKey"))
            .ok_or("member key")?,
        model_key: resolver
            .typed_field(ModelKey::at("ModelKey"))
            .ok_or("model key")?,
    })
}

/// The touched records of one commit, as scope planning admitted them.
pub struct Scope {
    members: Vec<Handle<Member>>,
    gone: Vec<Handle<Member>>,
    models: Vec<Handle<Model>>,
}

impl WorthQueryApplicationInvariantRule<Schema> for Rule {
    type Refusal = Refusal;
    type RefusalBinding = RefusalBinding;
    type Scope = Scope;
    fn prepare_scope(
        &self,
        planner: &mut WorthQueryApplicationInvariantScopePlanner<'_, '_, Schema>,
    ) -> Result<Scope, Error> {
        let (proposed, committed) = (planner.proposed(), planner.committed());
        let members = proposed.touched_entities_of(&self.member)?;
        let mut gone = Vec::new();
        for entity in committed.touched_entities_of(&self.member)? {
            if proposed.reobserve_entity(&self.member, &entity)?.is_none() {
                committed.field(&self.member_key, &entity)?;
                gone.push(entity);
            }
        }
        let models = proposed.touched_entities_of(&self.model)?;
        for member in &members {
            let seen = self.member(&proposed, &committed, member, Phase::Planned)?;
            self.log(seen);
        }
        for model in &models {
            proposed.field(&self.model_key, model)?;
        }
        Ok(Scope {
            members,
            gone,
            models,
        })
    }

    fn evaluate(
        &self,
        context: &WorthQueryApplicationInvariantContext<'_, '_, Schema>,
        scope: &Scope,
    ) -> Result<WorthQueryApplicationInvariantVerdict<Refusal>, Error> {
        let (proposed, committed) = (context.proposed(), context.committed());
        let forbidden = self.watch.lock().unwrap().forbidden_side;
        let mut verdict = WorthQueryApplicationInvariantVerdict::Pass;
        for member in &scope.members {
            let seen = self.member(&proposed, &committed, member, Phase::Evaluated)?;
            if let Seen::Member {
                key,
                birth: Some(Ok(birth)),
                ..
            } = &seen
                && let Ok(Some(MemberCut::Side { side, .. })) =
                    birth.decode_discriminator::<MemberCut>()
                && Some(side) == forbidden
            {
                verdict =
                    WorthQueryApplicationInvariantVerdict::Violation(Refusal::ForbiddenSide {
                        key: key.clone(),
                        side,
                    });
            }
            self.log(seen);
        }
        for entity in &scope.gone {
            let key = committed
                .field(&self.member_key, entity)?
                .unwrap_or_default();
            let seen = Seen::Gone {
                key,
                committed: committed.name_of(entity),
                reobserved: proposed.reobserve_entity(&self.member, entity)?.is_some(),
                cross_view: debug(proposed.name_of(entity).unwrap_err()),
            };
            self.log(seen);
        }
        for model in &scope.models {
            let key = proposed.field(&self.model_key, model)?.unwrap_or_default();
            let name = match proposed.name_of(model) {
                Ok(name) => format!("named {name:?}"),
                Err(denial) => debug(denial),
            };
            self.log(Seen::Model { key, name });
        }
        Ok(verdict)
    }
}

impl Rule {
    fn log(&self, seen: Seen) {
        self.watch.lock().unwrap().seen.push(seen);
    }

    /// One touched member's names in both views of the commit.
    fn member(
        &self,
        proposed: &WorthQueryApplicationInvariantReadView<'_, Schema>,
        committed: &WorthQueryApplicationInvariantReadView<'_, Schema>,
        member: &Handle<Member>,
        phase: Phase,
    ) -> Result<Seen, Error> {
        let key = proposed
            .field(&self.member_key, member)?
            .unwrap_or_default();
        let name = proposed.name_of(member);
        let birth = name.as_ref().ok().map(|name| proposed.birth_of(name));
        let committed_birth = name.as_ref().ok().map(|name| committed.birth_of(name));
        let committed = committed
            .reobserve_entity(&self.member, member)?
            .map(|before| committed.name_of(&before));
        Ok(Seen::Member {
            phase,
            key,
            proposed: name,
            birth,
            committed,
            committed_birth,
        })
    }
}

fn debug(value: impl std::fmt::Debug) -> String {
    format!("{value:?}")
}

/// A plan creating `keys` and naming each authored under its own occurrence.
pub fn authored(keys: &[&str]) -> Plan {
    Plan {
        creates: outputs::keys(keys),
        steps: keys
            .iter()
            .map(|key| Step::Birth((*key).into(), outputs::birth(key, "authored")))
            .collect(),
        ..Plan::default()
    }
}
