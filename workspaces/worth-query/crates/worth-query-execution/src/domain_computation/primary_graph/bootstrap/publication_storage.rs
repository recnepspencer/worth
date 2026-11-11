//! Prepaid backing lets bootstrap phases retire before their successors run.
use super::{
    WorthQueryPrimaryGraphInstallationDenial as Denial,
    WorthQueryPrimaryGraphInstallationDenialKind as Kind,
};
use std::cell::OnceCell;
use worth_execution::{
    ExecutionArray, ExecutionRequest, ExecutionRequestArrayBuilder, ExecutionRequestArrayDenial,
};

pub(in crate::domain_computation::primary_graph) struct BootstrapPublicationStorage<T> {
    values: ExecutionArray<OnceCell<T>>,
}
pub(in crate::domain_computation::primary_graph) struct BootstrapPublicationReservation<T> {
    values: ExecutionArray<OnceCell<T>>,
}
impl<T> BootstrapPublicationReservation<T> {
    pub(in crate::domain_computation::primary_graph) fn prepare(
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, Denial> {
        let mut builder = ExecutionRequestArrayBuilder::allocate(*request, 1).map_err(denied)?;
        builder.push(OnceCell::new()).map_err(denied)?;
        Ok(Self {
            values: builder.seal().map_err(denied)?,
        })
    }
    pub(in crate::domain_computation::primary_graph) fn fill(
        self,
        value: T,
    ) -> BootstrapPublicationStorage<T> {
        assert!(
            self.values.elements()[0].set(value).is_ok(),
            "bootstrap successor filled once"
        );
        BootstrapPublicationStorage {
            values: self.values,
        }
    }
}
impl<T> BootstrapPublicationStorage<T> {
    pub(in crate::domain_computation::primary_graph) fn value(&self) -> &T {
        self.values.elements()[0]
            .get()
            .expect("actual bootstrap successor")
    }
    pub(in crate::domain_computation::primary_graph) fn consume(self) -> T {
        self.values
            .into_iter()
            .next()
            .expect("one prepaid bootstrap cell")
            .into_inner()
            .expect("actual bootstrap successor")
    }
}
fn denied(cause: ExecutionRequestArrayDenial) -> Denial {
    match cause {
        ExecutionRequestArrayDenial::Request(cause) => {
            match crate::domain_computation::primary_graph::application_contribution::publication_execution_kind(cause) {
                Ok(kind) => Denial::new(Kind::ExecutionDenied { kind }, "bootstrap publication backing refused"),
                Err(kind) => Denial::new(Kind::ExecutionControlStopped { kind }, "bootstrap publication backing stopped"),
            }
        }
        ExecutionRequestArrayDenial::Allocation(cause) => {
            let failure = crate::domain_computation::WorthQueryProviderSessionFailure::allocation_denied(cause);
            Denial::new(Kind::ExecutionDenied { kind: failure.kind() }, failure.detail())
        }
    }
}
