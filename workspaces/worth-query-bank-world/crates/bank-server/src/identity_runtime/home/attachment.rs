//! Attach Bank command and workflow owners to the actual fresh or readmitted Home.
use super::*;
use worth_query_host::facade::{WorthQueryApplicationHome, WorthQueryApplicationHomeOpenRefused};

pub enum BankHomeOpenRefused {
    Preparation(BankIdentityRuntimeBuildError),
    Advancement(worth_query_host::facade::application_contribution::WorthQueryAdvancementDenial),
    Open(Box<WorthQueryApplicationHomeOpenRefused<BankSchema, BankApplication>>),
    Attachment {
        home: WorthQueryApplicationHome<BankSchema, BankApplication>,
        cause: BankIdentityRuntimeBuildError,
    },
}

/// The same recipe opens a fresh or existing Home. Only Home may invoke its seed.
pub fn open_bank_application_home(
    location: WorthQueryApplicationHomeLocation,
    seed: BankWorldSeed,
) -> Result<BankIdentityRuntime, BankHomeOpenRefused> {
    let index = std::sync::Arc::new(
        crate::application_definition::current_accounts::AccountIndexWork::default(),
    );
    let request = bank_application_home_with_index(location, seed, index.clone())
        .map_err(BankHomeOpenRefused::Preparation)?;
    let home = worth_query_host::facade::application_contribution::with_bootstrap_advancement(
        super::super::product_world_resources::bank_bootstrap_policy(),
        |phase| request.open_in_advancement(&phase),
    )
    .map_err(BankHomeOpenRefused::Advancement)?
    .map_err(|refusal| BankHomeOpenRefused::Open(Box::new(refusal)))?;
    attach(home, index)
}

fn attach(
    home: WorthQueryApplicationHome<BankSchema, BankApplication>,
    index: std::sync::Arc<crate::application_definition::current_accounts::AccountIndexWork>,
) -> Result<BankIdentityRuntime, BankHomeOpenRefused> {
    let prepared = (|| {
        let program = home.runtime();
        let schema = program.runtime().installed_schema();
        let binding = installation::resolve_installed_principal_binding(program.runtime())?;
        let approval = crate::approval_authentication::install_approval_authentication(
            schema,
            BankApprovalAuthenticationConfiguration::denying(),
        )
        .map_err(BankIdentityRuntimeBuildError::WorkflowAuthenticationInstallation)?;
        let workflow =
            installation::install_payment_workflow_spec(schema, program.installed_program())
                .map_err(BankIdentityRuntimeBuildError::WorkflowInstallation)?;
        Ok((binding, approval, workflow))
    })();
    let (binding, approval, workflow) = match prepared {
        Ok(prepared) => prepared,
        Err(cause) => return Err(BankHomeOpenRefused::Attachment { home, cause }),
    };
    let projection = home.retain_invariant_projection_authority();
    super::super::BankIdentityRuntimeWorkflowBinding::new(
        home, workflow, binding, projection, approval, index,
    )
    .bind()
    .map_err(BankHomeOpenRefused::Preparation)
}

impl std::fmt::Debug for BankHomeOpenRefused {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Preparation(cause) => f.debug_tuple("Preparation").field(cause).finish(),
            Self::Advancement(cause) => f.debug_tuple("Advancement").field(cause).finish(),
            Self::Open(cause) => f.debug_tuple("Open").field(cause).finish(),
            Self::Attachment { cause, .. } => f
                .debug_struct("Attachment")
                .field("cause", cause)
                .finish_non_exhaustive(),
        }
    }
}
impl std::fmt::Display for BankHomeOpenRefused {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Bank Home opening refused: {self:?}")
    }
}
impl std::error::Error for BankHomeOpenRefused {}
