//! The application's explicit ownership of a newly forked mutable component.
use super::super::{authentication::Caller, failure::Failure, host::Host};
use worth_query_host::facade::application_contribution::WorthQueryAdvancementDenial;
use worth_query_host::facade::product::{
    WorthQueryProductBranch, WorthQueryProductBranchCloseReceipt,
    WorthQueryProductBranchCreateError,
};

pub struct OwnedBranch<'host> {
    pub(super) host: &'host Host,
    pub(super) head: WorthQueryProductBranch,
}
impl std::fmt::Debug for OwnedBranch<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OwnedBranch")
            .field("head", &self.head)
            .finish()
    }
}
impl<'host> OwnedBranch<'host> {
    pub fn fork(
        host: &'host Host,
        source: WorthQueryProductBranch,
        caller: &Caller<'_>,
    ) -> Result<Self, ForkFailure<'host>> {
        let mut created = None;
        let advancement = host.with_application_advancement(caller.scope, |phase| {
            created = Some(
                host.branches()
                    .fork(source)
                    .components(|parts| parts.fork_relational().reuse_exact_signal_basis())
                    .create(&phase),
            );
        });
        match (created, advancement) {
            (Some(Ok(head)), Ok(())) => Ok(Self { host, head }),
            (Some(Ok(head)), Err(handoff)) => Err(ForkFailure::Created {
                owner: Self { host, head },
                handoff,
            }),
            (Some(Err(cause)), handoff) => Err(ForkFailure::Creation {
                cause,
                handoff: handoff.err(),
            }),
            (None, Err(cause)) => Err(ForkFailure::Advancement(cause)),
            (None, Ok(())) => unreachable!("successful advancement invokes its body"),
        }
    }
    pub fn branch(&self) -> WorthQueryProductBranch {
        self.head
    }

    pub fn close(self, caller: &Caller<'_>) -> Result<Closed, CloseFailure<'host>> {
        let mut closed = None;
        let advancement = self
            .host
            .with_application_advancement(caller.scope, |phase| {
                closed = Some(self.host.on_branch(self.head).close(&phase));
            });
        match (closed, advancement) {
            (Some(Ok(receipt)), handoff) => Ok(Closed {
                receipt,
                handoff: handoff.err(),
            }),
            (Some(Err(cause)), handoff) => Err(CloseFailure {
                owner: self,
                cause: Failure::Close(cause),
                handoff: handoff.err(),
            }),
            (None, Err(cause)) => Err(CloseFailure {
                owner: self,
                cause: Failure::Advancement(cause),
                handoff: None,
            }),
            (None, Ok(())) => unreachable!("successful advancement invokes its body"),
        }
    }
}

#[derive(Debug)]
pub struct CloseFailure<'host> {
    pub owner: OwnedBranch<'host>,
    pub cause: Failure,
    pub handoff: Option<WorthQueryAdvancementDenial>,
}
#[derive(Debug)]
pub struct Closed {
    pub receipt: WorthQueryProductBranchCloseReceipt,
    pub handoff: Option<WorthQueryAdvancementDenial>,
}
#[derive(Debug)]
pub enum ForkFailure<'host> {
    Advancement(WorthQueryAdvancementDenial),
    Creation {
        cause: WorthQueryProductBranchCreateError,
        handoff: Option<WorthQueryAdvancementDenial>,
    },
    Created {
        owner: OwnedBranch<'host>,
        handoff: WorthQueryAdvancementDenial,
    },
}
#[derive(Debug)]
pub struct InitializationFailure<'host> {
    pub owner: OwnedBranch<'host>,
    pub cause: Failure,
}
