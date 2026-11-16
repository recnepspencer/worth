//! Process failures retain the genuine listener, mutation or Home close owner.
use std::sync::Arc;

use crate::{
    AuthentikBankIdentity, BankHttpMutationContinuation, BankHttpServer, BankHttpServerClose,
    BankHttpServerCloseFailure,
};

pub enum BankHttpProcessCloseRefused {
    Installation {
        identity: Arc<AuthentikBankIdentity>,
        cause: std::io::Error,
    },
    Running {
        identity: Arc<AuthentikBankIdentity>,
        server: BankHttpServer,
        mutations: BankHttpMutationContinuation,
        cause: Box<dyn std::error::Error + Send + Sync>,
    },
    Workers {
        identity: Arc<AuthentikBankIdentity>,
        mutations: BankHttpMutationContinuation,
        failure: BankHttpServerCloseFailure,
    },
    Mutations {
        identity: Arc<AuthentikBankIdentity>,
        mutations: BankHttpMutationContinuation,
        close: BankHttpServerClose,
    },
    IdentityInUse(Arc<AuthentikBankIdentity>),
    Memory(bank_server::BankIdentityRuntime),
    Home(
        worth_query_host::facade::WorthQueryWorkflowHomeCloseRefused<
            bank_domain::schema::BankSchema,
            bank_domain::schema::ApprovedBusinessPaymentWorkflow,
            bank_server::BankApplication,
        >,
    ),
}

pub(super) async fn finish(
    identity: Arc<AuthentikBankIdentity>,
    mutations: BankHttpMutationContinuation,
    close: BankHttpServerClose,
    request: &worth_execution::ExecutionRequest<'_, '_>,
) -> Result<(), BankHttpProcessCloseRefused> {
    if let Err(mutations) = mutations.finish().await {
        return Err(BankHttpProcessCloseRefused::Mutations {
            identity,
            mutations,
            close,
        });
    }
    // All listener and mutation workers are joined. Release their local callback
    // route at the graceful handoff; pending rail provenance remains World custody.
    drop(close);
    let identity = Arc::try_unwrap(identity).map_err(BankHttpProcessCloseRefused::IdentityInUse)?;
    let home = identity
        .into_runtime()
        .into_home()
        .map_err(BankHttpProcessCloseRefused::Memory)?;
    home.close(request)
        .map_err(BankHttpProcessCloseRefused::Home)?
        .reclaim();
    Ok(())
}

impl std::fmt::Debug for BankHttpProcessCloseRefused {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self, f)
    }
}
impl std::fmt::Display for BankHttpProcessCloseRefused {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Installation { cause, .. } => write!(f, "Bank listener installation: {cause}"),
            Self::Running { cause, .. } => write!(f, "Bank process retained live service: {cause}"),
            Self::Workers { failure, .. } => failure.fmt(f),
            Self::Mutations { .. } => {
                f.write_str("Bank mutation worker finish refused; continuation retained")
            }
            Self::IdentityInUse(_) => f.write_str("Bank Home still has an HTTP identity borrower"),
            Self::Memory(_) => f.write_str("Bank process requires an owned Directory Home"),
            Self::Home(refusal) => write!(f, "Bank Home close refused: {refusal:?}"),
        }
    }
}
impl std::error::Error for BankHttpProcessCloseRefused {}
