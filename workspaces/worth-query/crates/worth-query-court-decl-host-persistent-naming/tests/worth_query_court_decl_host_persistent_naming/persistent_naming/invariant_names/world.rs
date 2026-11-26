//! The watched world: the shared persistent-naming program plus `NameWatch`
//! at the commit boundary, on its own installed program and with its own log.
use super::*;
use crate::persistent_naming::world::{member_identity, try_install_program};
use worth_query_host::facade::runtime::ExecutionAllocationPolicy;

pub struct WatchProgram;
impl ApplicationProgramDefinition<Schema> for WatchProgram {
    type Contributions = (Contribution,);
    type Outputs = ApplicationProgramOutputs<coupled::producer::Graph>;
    type Rules = ApplicationRuleList<
        ApplicationRuleAt<
            ApplicationLocalRuleRef<Schema, Entry, NameWatch>,
            ApplicationCommitBoundary,
        >,
        ApplicationRuleLeaf,
    >;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("persistent-naming.name-watch-program.v1");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        // The shared program's features: only the rule list differs.
        <crate::persistent_naming::Program as ApplicationProgramDefinition<Schema>>::feature_specs()
    }
}

pub type Host = WorthQueryProgramApplicationRuntime<Schema, WatchProgram>;
pub type DocumentOf<'a> = WorthQueryApplicationHistoryDocument<'a, Schema, History>;

/// One live watched world, its operator's request and its watch.
pub struct WatchWorld {
    pub host: Host,
    scope: worth_query_host::facade::admission::authenticated_principal::WorthQueryRequestScope,
    principal: WorthQueryAuthenticatedExternalPrincipal<Schema>,
    watch: Shared,
}

impl WatchWorld {
    pub fn install() -> Self {
        let watch = Shared::default();
        INSTALLING.with(|slot| *slot.borrow_mut() = Some(Arc::clone(&watch)));
        let host =
            try_install_program::<WatchProgram>(true, "91820000-0000-0000-0000-0000000a7418");
        INSTALLING.with(|slot| *slot.borrow_mut() = None);
        let host = host.unwrap();
        let scope = request_scope();
        let principal = authenticate_operator(host.installed_schema(), &scope);
        Self {
            host,
            scope,
            principal,
            watch,
        }
    }

    fn request(&self) -> WorthQueryApplicationRequest<'_, '_, '_, Schema> {
        self.host.request(&self.principal, &self.scope)
    }

    /// Refuses, from now on, a proposed birth decoding cut side `side`.
    pub fn forbid_side(&self, side: u64) {
        self.watch.lock().unwrap().forbidden_side = Some(side);
    }

    /// Everything the watch saw since the last call.
    pub fn take(&self) -> Vec<Seen> {
        std::mem::take(&mut self.watch.lock().unwrap().seen)
    }

    /// Runs `plan` as one installed sculpt operation.
    pub fn sculpt(&self, token: &str, plan: Plan) -> outputs::Outcome {
        outputs::sculpt::author(token, plan);
        self.request()
            .advanced()
            .mutate(outputs::sculpt::Intent {
                token: token.to_owned(),
            })
            .without_source()
            .idempotency(&format!("sculpt:{token}"))
            .execute_in_program(&self.host, ExecutionAllocationPolicy::SystemAllocation)
            .unwrap_or_else(|cause| panic!("{token}: {cause:?}"))
    }

    pub fn committed(&self, token: &str, plan: Plan) {
        let outcome = self.sculpt(token, plan);
        assert!(
            matches!(outcome, outputs::Outcome::Committed { .. }),
            "{token}: {outcome:?}"
        );
    }

    /// The names an independent tracked reader reads for `keys` now.
    pub fn names(&self, token: &str, keys: &[&str], ordinal: u64) -> Vec<WorthQueryName<Member>> {
        let _ = self
            .request()
            .advanced()
            .mutate(probe::Intent {
                token: token.to_owned(),
                keys: outputs::keys(keys),
                value: 200 + ordinal,
            })
            .without_source()
            .idempotency(&format!("probe:{token}:{ordinal}"))
            .execute_in_program(&self.host, ExecutionAllocationPolicy::SystemAllocation);
        let observed = probe::take(token);
        keys.iter()
            .map(|key| outputs::named(&observed, key))
            .collect()
    }

    /// The births an independent tracked decision reader reads for `names`
    /// now, keyed by label.
    pub fn births(
        &self,
        token: &str,
        names: &[(&str, WorthQueryName<Member>)],
        ordinal: u64,
    ) -> Vec<RecordedNameBirth<Member>> {
        for (label, name) in names {
            probe::births::birth_of(token, label, *name);
        }
        let _ = self
            .request()
            .advanced()
            .mutate(probe::Intent {
                token: token.to_owned(),
                keys: Vec::new(),
                value: 400 + ordinal,
            })
            .without_source()
            .idempotency(&format!("birth-probe:{token}:{ordinal}"))
            .execute_in_program(&self.host, ExecutionAllocationPolicy::SystemAllocation);
        probe::take(token);
        let observed = probe::births::take(token);
        names
            .iter()
            .map(|(label, _)| {
                observed
                    .iter()
                    .find_map(|seen| match seen {
                        probe::births::BirthObservation::Born { label: l, birth } if l == label => {
                            Some(birth.clone())
                        }
                        _ => None,
                    })
                    .unwrap_or_else(|| panic!("`{label}` must read its birth: {observed:?}"))
            })
            .collect()
    }

    pub fn live(&self, key: &str) -> bool {
        member_identity(&self.host, self.host.current_world(), key).is_some()
    }

    /// Edits member `key`'s value: the member is touched, its name kept.
    pub fn edit_member(&self, key: &str, value: u64, token: &str) {
        let edit = edit::Edit {
            key: key.to_owned(),
            value,
        };
        let outcome = self
            .request()
            .advanced()
            .mutate(edit::member::Intent(edit))
            .without_source()
            .idempotency(&token.to_owned())
            .execute_in_program(&self.host, ExecutionAllocationPolicy::SystemAllocation)
            .unwrap();
        assert!(
            matches!(outcome, outputs::Outcome::Committed { .. }),
            "{outcome:?}"
        );
    }

    /// Edits model `key`'s value; no producer runs.
    pub fn edit_model(&self, key: &str, value: u64, token: &str) {
        let edit = edit::Edit {
            key: key.to_owned(),
            value,
        };
        let outcome = self
            .request()
            .advanced()
            .mutate(edit::model::Intent(edit))
            .without_source()
            .idempotency(&token.to_owned())
            .execute_in_program(&self.host, ExecutionAllocationPolicy::SystemAllocation)
            .unwrap();
        assert!(
            matches!(outcome, outputs::Outcome::Committed { .. }),
            "{outcome:?}"
        );
    }

    /// Edits model `key`; its producer names its outputs in the same envelope.
    pub fn shape(&self, key: &str, value: u64, token: &str) -> outputs::Outcome {
        self.request()
            .advanced()
            .mutate(coupled::producer::shape::Intent {
                key: key.to_owned(),
                value,
                collide: false,
            })
            .without_source()
            .idempotency(&token.to_owned())
            .execute_in_program(&self.host, ExecutionAllocationPolicy::SystemAllocation)
            .unwrap_or_else(|cause| panic!("{token}: {cause:?}"))
    }
}
