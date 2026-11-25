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
    application_query::ApplicationManagedIndexBinding, application_schema::ApplicationSchema,
};
use worth_query_installation::facade::WorthQueryInstalledApplicationValueBindingCatalog;

/// A descriptive source slot issued by the selected source owner. It is not an entity handle.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct WorthQueryManagedIndexSourceSlot {
    pub(crate) owner: WorthQueryGraphWorkSessionIdentity,
    pub(crate) ordinal: usize,
    pub(crate) entity: Option<worth_relational::facade::identity::EntityId>,
}

#[derive(Clone, Copy)]
pub enum SourceImageRows<'a> {
    Completed {
        tree: &'a WorthQueryApplicationDisclosedProjectionTree,
        order: &'a [usize],
    },
    Maintained(&'a super::source::occurrences::PersistentSourceRows),
}

/// Borrowed only from the private completed source owner, never from application values.
#[derive(Clone, Copy)]
pub struct SourceImageView<'a> {
    pub owner: WorthQueryGraphWorkSessionIdentity,
    pub rows: SourceImageRows<'a>,
    pub governance: &'a WorthQueryApplicationQueryGovernance,
    pub bindings: &'a WorthQueryInstalledApplicationValueBindingCatalog,
    pub source_budget: Option<&'a worth_relational::facade::runtime::CustomInvariantReadBudget<'a>>,
}

/// Depth-proportional work charge for one ordered-map lookup over `len` maintained rows.
fn maintained_lookup_work(len: usize) -> u64 {
    u64::from((usize::BITS - len.leading_zeros()).max(1))
}

/// Query-bound source rows borrowed during one controlled provider invocation.
pub struct WorthQueryManagedIndexSourceRows<
    'a,
    S: ApplicationSchema,
    I: ApplicationManagedIndexBinding<S>,
> {
    image: SourceImageView<'a>,
    text: super::source_text::SourceText,
    execution: worth_execution::ExecutionRequest<'a, 'a>,
    decode: &'a ValueDecodeSession<'a>,
    control: &'a WorthQueryManagedIndexControl<'a>,
    marker: std::marker::PhantomData<fn() -> (S, I)>,
}

impl<'a, S: ApplicationSchema, I: ApplicationManagedIndexBinding<S>>
    WorthQueryManagedIndexSourceRows<'a, S, I>
{
    pub(crate) fn new(
        image: SourceImageView<'a>,
        decode: &'a ValueDecodeSession<'a>,
        control: &'a WorthQueryManagedIndexControl<'a>,
        execution: worth_execution::ExecutionRequest<'a, 'a>,
    ) -> Self {
        Self {
            image,
            text: super::source_text::SourceText::default(),
            execution,
            decode,
            control,
            marker: std::marker::PhantomData,
        }
    }
    pub fn len(&self) -> usize {
        match self.image.rows {
            SourceImageRows::Completed { order, .. } => order.len(),
            SourceImageRows::Maintained(rows) => rows.len(),
        }
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
            super::WorthQueryManagedIndexSourceRow<'_, S, I>,
        ),
        Stop,
    > {
        self.control.checkpoint(1)?;
        let (node, entity) = match self.image.rows {
            SourceImageRows::Completed { tree, order } => {
                let actual = order
                    .get(ordinal)
                    .ok_or_else(|| self.control.refuse(Stop::SourceSlotUnavailable))?;
                (
                    tree.at(*actual)
                        .ok_or_else(|| self.control.refuse(Stop::SourceSlotUnavailable))?,
                    None,
                )
            }
            SourceImageRows::Maintained(rows) => {
                // Charge ordered-map descent depth before resolving canonical rank.
                self.control
                    .checkpoint(maintained_lookup_work(rows.len()))?;
                let (entity, node) = rows
                    .at(ordinal)
                    .ok_or_else(|| self.control.refuse(Stop::SourceSlotUnavailable))?;
                (node, Some(entity))
            }
        };
        let slot = WorthQueryManagedIndexSourceSlot {
            owner: self.image.owner,
            ordinal: if entity.is_some() { 0 } else { ordinal },
            entity,
        };
        Ok((slot, self.row(slot, node)))
    }
    pub fn get(
        &self,
        slot: WorthQueryManagedIndexSourceSlot,
    ) -> Result<super::WorthQueryManagedIndexSourceRow<'_, S, I>, Stop> {
        if slot.owner != self.image.owner {
            return Err(self.control.refuse(Stop::ForeignSourceSlot));
        }
        match (self.image.rows, slot.entity) {
            (SourceImageRows::Maintained(rows), Some(entity)) if slot.ordinal == 0 => {
                self.control
                    .checkpoint(maintained_lookup_work(rows.len()))?;
                let node = rows
                    .get(entity)
                    .ok_or_else(|| self.control.refuse(Stop::SourceSlotUnavailable))?;
                Ok(self.row(slot, node))
            }
            (SourceImageRows::Completed { .. }, None) => self.at(slot.ordinal).map(|(_, row)| row),
            _ => Err(self.control.refuse(Stop::SourceSlotUnavailable)),
        }
    }
    fn row<'row>(
        &'row self,
        slot: WorthQueryManagedIndexSourceSlot,
        node: super::super::projection::WorthQueryApplicationDisclosedProjectionNode<'row>,
    ) -> super::WorthQueryManagedIndexSourceRow<'row, S, I> {
        super::WorthQueryManagedIndexSourceRow::new(
            WorthQueryApplicationProjectionRow::new(node, self.image.governance, self.decode),
            slot,
            &self.text,
            self.decode,
            self.control,
            self.execution,
        )
    }
    pub(crate) fn image(&self) -> SourceImageView<'a> {
        self.image
    }
    /// Enumerate cold material without repeated rank lookups. Coordinates of
    /// maintained occurrences remain stable across insertions and retirements.
    pub fn slots(
        &self,
    ) -> impl Iterator<Item = Result<WorthQueryManagedIndexSourceSlot, Stop>> + '_ {
        let mut ordinal = 0;
        let mut keys = match self.image.rows {
            SourceImageRows::Maintained(rows) => Some(rows.keys()),
            SourceImageRows::Completed { .. } => None,
        };
        std::iter::from_fn(move || {
            let (position, entity) = if let Some(keys) = &mut keys {
                (0, Some(keys.next()?))
            } else {
                if ordinal == self.len() {
                    return None;
                }
                let position = ordinal;
                ordinal += 1;
                (position, None)
            };
            Some(
                self.control
                    .checkpoint(1)
                    .map(|()| WorthQueryManagedIndexSourceSlot {
                        owner: self.image.owner,
                        ordinal: position,
                        entity,
                    }),
            )
        })
    }
    pub(crate) fn check(&self) -> Result<(), Stop> {
        if let Some(cause) = self.decode.denial() {
            return Err(self.control.refuse(Stop::ValueDecode(cause)));
        }
        self.control.poll()
    }
}

impl SourceImageView<'_> {
    pub fn same_image(&self, other: &Self) -> bool {
        if self.owner != other.owner || !std::ptr::eq(self.bindings, other.bindings) {
            return false;
        }
        match (self.rows, other.rows) {
            (
                SourceImageRows::Completed {
                    tree: left,
                    order: a,
                },
                SourceImageRows::Completed {
                    tree: right,
                    order: b,
                },
            ) => std::ptr::eq(left, right) && std::ptr::eq(a, b),
            (SourceImageRows::Maintained(left), SourceImageRows::Maintained(right)) => {
                std::ptr::eq(left, right)
            }
            _ => false,
        }
    }
    pub fn len(&self) -> usize {
        match self.rows {
            SourceImageRows::Completed { order, .. } => order.len(),
            SourceImageRows::Maintained(rows) => rows.len(),
        }
    }
    pub fn validate_slot(
        &self,
        slot: WorthQueryManagedIndexSourceSlot,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<(), Stop> {
        if slot.owner != self.owner {
            return Err(control.refuse(Stop::ForeignSourceSlot));
        }
        match (self.rows, slot.entity) {
            (SourceImageRows::Maintained(rows), Some(entity)) if slot.ordinal == 0 => {
                control.checkpoint(maintained_lookup_work(rows.len()))?;
                if rows.contains(entity) {
                    Ok(())
                } else {
                    Err(control.refuse(Stop::SourceSlotUnavailable))
                }
            }
            (SourceImageRows::Completed { .. }, None) if slot.ordinal < self.len() => Ok(()),
            _ => Err(control.refuse(Stop::SourceSlotUnavailable)),
        }
    }
}
