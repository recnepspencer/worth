//! Borrowed rows and slots from one genuine completed-source session.
use super::{WorthQueryManagedIndexControl, WorthQueryManagedIndexStop as Stop};
use crate::domain_computation::primary_graph::{
    application_query::{
        disclosure::WorthQueryApplicationQueryGovernance,
        projection::{
            WorthQueryApplicationDisclosedProjectionTree, WorthQueryApplicationProjectionRow,
        },
    },
    value_decode::ValueDecodeSession,
};
use crate::domain_computation::provider_session::WorthQueryGraphWorkSessionIdentity;
use worth_query_declaration::facade::{
    application_query::{ApplicationManagedIndexBinding, ApplicationQueryBinding},
    application_schema::ApplicationSchema,
};
use worth_query_installation::facade::WorthQueryInstalledApplicationValueBindingCatalog;

/// A descriptive source slot issued by the selected source owner. It is not an entity handle.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryManagedIndexSourceSlot {
    pub(super) owner: WorthQueryGraphWorkSessionIdentity,
    pub(super) ordinal: usize,
}

/// Borrowed only from the private completed source owner, never from application values.
#[derive(Clone, Copy)]
pub(super) struct SourceImageView<'a> {
    pub(super) owner: WorthQueryGraphWorkSessionIdentity,
    pub(super) tree: &'a WorthQueryApplicationDisclosedProjectionTree,
    pub(super) order: &'a [usize],
    pub(super) governance: &'a WorthQueryApplicationQueryGovernance,
    pub(super) bindings: &'a WorthQueryInstalledApplicationValueBindingCatalog,
}

/// Query-bound source rows borrowed during one controlled provider invocation.
pub struct WorthQueryManagedIndexSourceRows<
    'a,
    S: ApplicationSchema,
    I: ApplicationManagedIndexBinding<S>,
> {
    image: SourceImageView<'a>,
    decode: &'a ValueDecodeSession<'a>,
    control: &'a WorthQueryManagedIndexControl<'a>,
    marker: std::marker::PhantomData<fn() -> (S, I)>,
}

impl<'a, S: ApplicationSchema, I: ApplicationManagedIndexBinding<S>>
    WorthQueryManagedIndexSourceRows<'a, S, I>
{
    pub(super) fn new(
        image: SourceImageView<'a>,
        decode: &'a ValueDecodeSession<'a>,
        control: &'a WorthQueryManagedIndexControl<'a>,
    ) -> Self {
        Self {
            image,
            decode,
            control,
            marker: std::marker::PhantomData,
        }
    }
    pub fn len(&self) -> usize {
        self.image.order.len()
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    pub fn at(
        &self,
        ordinal: usize,
    ) -> Result<
        (
            WorthQueryManagedIndexSourceSlot,
            WorthQueryApplicationProjectionRow<
                '_,
                S,
                <I::SourceBinding as ApplicationQueryBinding<S>>::Query,
            >,
        ),
        Stop,
    > {
        self.control.checkpoint(1)?;
        let actual = self
            .image
            .order
            .get(ordinal)
            .ok_or_else(|| self.control.refuse(Stop::SourceSlotUnavailable))?;
        let node = self
            .image
            .tree
            .at(*actual)
            .ok_or_else(|| self.control.refuse(Stop::SourceSlotUnavailable))?;
        Ok((
            WorthQueryManagedIndexSourceSlot {
                owner: self.image.owner,
                ordinal,
            },
            WorthQueryApplicationProjectionRow::new(node, self.image.governance, self.decode),
        ))
    }
    pub fn get(
        &self,
        slot: WorthQueryManagedIndexSourceSlot,
    ) -> Result<
        WorthQueryApplicationProjectionRow<
            '_,
            S,
            <I::SourceBinding as ApplicationQueryBinding<S>>::Query,
        >,
        Stop,
    > {
        self.validate(slot)?;
        self.at(slot.ordinal).map(|(_, row)| row)
    }
    pub(super) fn owner(&self) -> WorthQueryGraphWorkSessionIdentity {
        self.image.owner
    }
    pub(super) fn check(&self) -> Result<(), Stop> {
        if let Some(cause) = self.decode.denial() {
            return Err(self.control.refuse(Stop::ValueDecode(cause)));
        }
        self.control.poll()
    }
    pub(super) fn validate(&self, slot: WorthQueryManagedIndexSourceSlot) -> Result<(), Stop> {
        if slot.owner != self.image.owner {
            return Err(self.control.refuse(Stop::ForeignSourceSlot));
        }
        if slot.ordinal >= self.len() {
            return Err(self.control.refuse(Stop::SourceSlotUnavailable));
        }
        Ok(())
    }
}
