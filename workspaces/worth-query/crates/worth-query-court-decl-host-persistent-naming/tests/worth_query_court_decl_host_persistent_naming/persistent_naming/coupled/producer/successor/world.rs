//! The successor-round world: the shared persistent-naming program plus the
//! `stir` edit, its two producers and its output tree, on its own installed
//! program. The shared program never installs these producers; the
//! contribution registers them only while this world installs (`installing`).
use super::*;
use crate::persistent_naming::world::try_install_program;
use std::cell::Cell;
use worth_query_decl::facade::application_naming::NameSelection;

thread_local! {
    static INSTALLING: Cell<bool> = const { Cell::new(false) };
}

/// Whether the installation running on this thread is the successor world's.
pub fn installing() -> bool {
    INSTALLING.with(Cell::get)
}

pub struct SuccessorProgram;
impl ApplicationProgramDefinition<Schema> for SuccessorProgram {
    type Contributions = (Contribution,);
    type Outputs = ApplicationProgramOutputs<(Graph, Tree)>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("persistent-naming.successor-program.v1");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        vec![
            crate::persistent_naming::entry()
                .action::<Binding, Delivery>()
                .finish_action()
                .finish(),
            super::super::feature(),
            root::feature(),
            child::feature(),
        ]
    }
}

/// One live successor world and its operator's request.
pub struct SuccessorWorld {
    host: WorthQueryProgramApplicationRuntime<Schema, SuccessorProgram>,
    scope: worth_query_host::facade::admission::authenticated_principal::WorthQueryRequestScope,
    principal: WorthQueryAuthenticatedExternalPrincipal<Schema>,
}

fn policy() -> worth_query_host::facade::runtime::ExecutionAllocationPolicy<'static, 'static> {
    worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation
}

impl SuccessorWorld {
    pub fn install() -> Self {
        INSTALLING.with(|installing| installing.set(true));
        let host =
            try_install_program::<SuccessorProgram>(true, "91820000-0000-0000-0000-00000000a71f");
        INSTALLING.with(|installing| installing.set(false));
        let host = host.unwrap();
        let scope = request_scope();
        let principal = authenticate_operator(host.installed_schema(), &scope);
        Self {
            host,
            scope,
            principal,
        }
    }

    fn request(&self) -> WorthQueryApplicationRequest<'_, '_, '_, Schema> {
        self.host.request(&self.principal, &self.scope)
    }

    /// Runs `plan` as one committed sculpt operation.
    pub fn committed(&self, token: &str, plan: Plan) {
        sculpt::author(token, plan);
        let outcome = self
            .request()
            .advanced()
            .mutate(sculpt::Intent {
                token: token.to_owned(),
            })
            .without_source()
            .idempotency(&format!("sculpt:{token}"))
            .execute_in_program(&self.host, policy());
        assert!(
            matches!(outcome, Ok(outputs::Outcome::Committed { .. })),
            "{token}: {outcome:?}"
        );
    }

    /// Runs one probe over `keys`; its observations survive a refusal.
    pub fn probe(&self, token: &str, keys: &[&str], ordinal: u64) -> Vec<probe::Observation> {
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
            .execute_in_program(&self.host, policy());
        probe::take(token)
    }

    pub fn name(&self, token: &str, key: &str) -> WorthQueryName<Member> {
        outputs::named(&self.probe(token, &[key], 0), key)
    }

    pub fn seam_name(&self, token: &str, from: &str, to: &str) -> WorthQueryName<Seam> {
        probe::seams::name(token, "read", from, to);
        self.probe(token, &[from, to], 0);
        outputs::seam_named(&probe::seams::take(token), "read")
    }

    /// Whether held `name` resolves `Retired` here.
    pub fn retired(&self, token: &str, name: WorthQueryName<Member>) -> bool {
        probe::hold(token, "held", name);
        let observed = self.probe(token, &[], 1);
        outputs::resolution(&observed, "held")
            == &probe::Observation::Retired {
                label: "held".into(),
            }
    }

    /// Whether held seam `name` resolves `Retired` here.
    pub fn seam_retired(&self, token: &str, name: WorthQueryName<Seam>) -> bool {
        probe::seams::hold(token, "held", name, NameSelection::Exact);
        self.probe(token, &[], 1);
        matches!(
            outputs::seam_resolution(&probe::seams::take(token), "held"),
            probe::seams::SeamObservation::Retired { .. }
        )
    }

    /// Edits model `key` to `value` as one envelope; both producers follow.
    pub fn stir(&self, idempotency: &str, key: &str, value: u64) -> Envelope {
        let mut envelope = self
            .request()
            .edit_envelope()
            .idempotency(idempotency.to_owned())
            .runtime_stages(&self.host)
            .ordered_authored()
            .unwrap();
        envelope
            .push_stage::<Tree, _>(Intent {
                key: key.to_owned(),
                value,
            })
            .unwrap();
        envelope.execute_performed(policy()).unwrap()
    }
}
