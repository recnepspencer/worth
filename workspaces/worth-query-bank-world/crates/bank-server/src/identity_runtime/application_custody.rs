#![allow(
    clippy::result_large_err,
    reason = "A refused binding hands back the original application, workflow spec and signing owner without allocating on failure"
)]
//! Bank commands borrow either the current in-memory installation or its owned Home.
use super::*;
use worth_query_host::facade::{
    admission::authentication_event::WorthQueryAuthenticationEventSigningOwner,
    application_installation::{
        WorthQueryWorkflowRuntimeBindingDenial, WorthQueryWorkflowVocabulary,
    },
    domain::WorthQueryInstalledApplicationWorkflowSpec,
    WorthQueryApplicationHome,
};

pub type BankWorkflowHome = WorthQueryApplicationHome<BankSchema, BankApplication>;

pub(crate) enum BankApplicationCustody {
    Memory(
        WorthQueryWorkflowApplicationRuntime<
            BankSchema,
            ApprovedBusinessPaymentWorkflow,
            BankApplication,
        >,
    ),
    Home(BankWorkflowHome),
}

pub(super) enum BankUnboundApplication {
    Memory(WorthQueryProgramApplicationRuntime<BankSchema, BankApplication>),
    Home(WorthQueryApplicationHome<BankSchema, BankApplication>),
}

impl From<WorthQueryProgramApplicationRuntime<BankSchema, BankApplication>>
    for BankUnboundApplication
{
    fn from(runtime: WorthQueryProgramApplicationRuntime<BankSchema, BankApplication>) -> Self {
        Self::Memory(runtime)
    }
}
impl From<WorthQueryApplicationHome<BankSchema, BankApplication>> for BankUnboundApplication {
    fn from(home: WorthQueryApplicationHome<BankSchema, BankApplication>) -> Self {
        Self::Home(home)
    }
}

impl BankUnboundApplication {
    pub(super) fn retain_workflow_spec(
        self,
        workflow: WorthQueryInstalledApplicationWorkflowSpec<
            BankSchema,
            ApprovedBusinessPaymentWorkflow,
        >,
        authentication: WorthQueryAuthenticationEventSigningOwner<BankSchema>,
    ) -> Result<
        BankApplicationCustody,
        (
            Self,
            WorthQueryInstalledApplicationWorkflowSpec<BankSchema, ApprovedBusinessPaymentWorkflow>,
            WorthQueryAuthenticationEventSigningOwner<BankSchema>,
            WorthQueryWorkflowRuntimeBindingDenial,
        ),
    > {
        match self {
            Self::Memory(runtime) => runtime
                .retain_workflow_spec(workflow, authentication)
                .map(BankApplicationCustody::Memory)
                .map_err(|refusal| {
                    let (runtime, workflow, authentication, denial) = refusal.into_parts();
                    (Self::Memory(runtime), workflow, authentication, denial)
                }),
            Self::Home(home) => home
                .retain_workflow_spec(workflow, authentication)
                .map(BankApplicationCustody::Home)
                .map_err(|refusal| {
                    let (home, workflow, authentication, denial) = refusal.into_parts();
                    (Self::Home(home), workflow, authentication, denial)
                }),
        }
    }
}

impl BankApplicationCustody {
    pub(crate) fn program_runtime(
        &self,
    ) -> &WorthQueryProgramApplicationRuntime<BankSchema, BankApplication> {
        match self {
            Self::Memory(runtime) => runtime.program_runtime(),
            Self::Home(home) => home.runtime(),
        }
    }

    pub(crate) fn workflow_spec(
        &self,
    ) -> &worth_query_host::facade::domain::WorthQueryInstalledApplicationWorkflowSpec<
        BankSchema,
        ApprovedBusinessPaymentWorkflow,
    > {
        match self {
            Self::Memory(runtime) => runtime.workflow_spec(),
            Self::Home(home) => home
                .retained_workflow_spec()
                .expect("Bank registered its payment workflow"),
        }
    }
}

impl<'a> From<&'a BankApplicationCustody>
    for WorthQueryWorkflowVocabulary<'a, BankSchema, ApprovedBusinessPaymentWorkflow>
{
    fn from(custody: &'a BankApplicationCustody) -> Self {
        match custody {
            BankApplicationCustody::Memory(runtime) => runtime.vocabulary(),
            BankApplicationCustody::Home(home) => home
                .workflow_vocabulary()
                .expect("Bank registered its payment workflow"),
        }
    }
}

impl BankIdentityRuntime {
    /// Release Bank authentication and projection borrows before closing the actual Home.
    /// A caller holding a Memory installation receives its original runtime back.
    pub fn into_home(self) -> Result<BankWorkflowHome, Self> {
        match self.runtime {
            BankApplicationCustody::Home(home) => Ok(home),
            BankApplicationCustody::Memory(_) => Err(self),
        }
    }
}
