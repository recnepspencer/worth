mod bundle_publication;
mod diagnostic_emission;
mod pending_settlement;
mod performed_settlement;
mod post_commit_consumer;
mod post_commit_consumption;
mod post_commit_diagnostics;
mod settlement_port;

use crate::runtime::RelationalRuntime;

pub struct PublicationAuthority<'runtime> {
    pub(super) runtime: &'runtime RelationalRuntime,
}

pub(crate) struct PublicationPreparationAuthority<'runtime> {
    pub(super) runtime: &'runtime crate::runtime::RelationalPreparationRuntime,
}

impl RelationalRuntime {
    pub(crate) fn publication_authority(&self) -> PublicationAuthority<'_> {
        PublicationAuthority::new(self)
    }
}

impl crate::runtime::RelationalPreparationRuntime {
    pub(crate) fn publication_preparation_authority(&self) -> PublicationPreparationAuthority<'_> {
        PublicationPreparationAuthority::new(self)
    }
}

impl<'runtime> PublicationPreparationAuthority<'runtime> {
    pub(crate) fn new(runtime: &'runtime crate::runtime::RelationalPreparationRuntime) -> Self {
        Self { runtime }
    }
}

impl<'runtime> PublicationAuthority<'runtime> {
    pub(crate) fn new(runtime: &'runtime RelationalRuntime) -> Self {
        Self { runtime }
    }
}

pub use settlement_port::RelationalSettlementPort;

pub(crate) use post_commit_consumer::production_post_commit_consumer;
pub use post_commit_consumer::{
    PostCommitConsumer, PostCommitConsumptionContext, PostCommitConsumptionFailure,
};
