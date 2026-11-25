//! Durable O(1) root pins on a serving Store (Store Milestone 4, Phase M4.1).
//!
//! `pin_current` selects the current root under the current-root lock and
//! reclaim fence, protects it in memory at once, and appends one constant-size
//! entry to the Store-owned `store.root-pins` journal: no record scan and no
//! copy of base state. Open replays that journal before the serving runtime is
//! handed out, so a pinned root and every ancestor a live child overlays stay
//! protected across displaced-segment retirement and crash/reopen, while
//! unrelated displaced garbage keeps retiring.
//!
//! Ordering keeps protection conservative across any crash: a pin takes effect
//! in memory before its durable entry, while `Expiring` and `Expired` steps
//! are written durably before memory advances (`Expiring` is also applied
//! conservatively in memory if its append fate is indeterminate, and left
//! unchanged on proven `NoEffect`).
pub(in crate::physical_runtime) mod event;
mod loan;
pub(in crate::physical_runtime) mod restore;

use crate::physical_runtime::{
    AdmittedRecordPlacementPolicy, OrderedJournalName, OrderedJournalPublication,
    OrderedJournalPublicationFailure, PhysicalMutationDeadline, ServingPhysicalRuntime,
};
use crate::pin_table::{RootPinDenial, RootPinDropPolicy, RootPinExpiry, RootPinId, RootPinState};
use event::{RootPinEvent, MAXIMUM_PIN_PAYLOAD_BYTES};
pub use loan::PhysicalRootPinLoan;

/// The journal holding Store's durable root pin entries.
pub const PHYSICAL_ROOT_PINS: &str = "store.root-pins";

/// A durable pin and the root generation it protects.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhysicalRootPinReceipt {
    pin: RootPinId,
    root_generation: u64,
}

impl PhysicalRootPinReceipt {
    pub const fn pin(self) -> RootPinId {
        self.pin
    }
    pub const fn root_generation(self) -> u64 {
        self.root_generation
    }
}

/// Where and how long each durable pin entry may take.
#[derive(Debug, Clone, Copy)]
pub struct PhysicalRootPinWrite {
    pub placement: AdmittedRecordPlacementPolicy,
    pub deadline: PhysicalMutationDeadline,
}

/// Borrowed pin door of one serving Store.
pub struct PhysicalRootPins<'runtime> {
    serving: &'runtime ServingPhysicalRuntime,
}

enum AppendFate {
    NoEffect,
    Indeterminate,
}

impl<'runtime> PhysicalRootPins<'runtime> {
    pub(in crate::physical_runtime) const fn new(
        serving: &'runtime ServingPhysicalRuntime,
    ) -> Self {
        Self { serving }
    }

    /// Pin the current root, optionally overlaying `parent`.
    ///
    /// Cost: one constant-size journal append; independent of Store size.
    pub fn pin_current(
        &self,
        parent: Option<RootPinId>,
        policy: RootPinDropPolicy,
        payload: Vec<u8>,
        write: PhysicalRootPinWrite,
    ) -> Result<PhysicalRootPinReceipt, RootPinDenial> {
        if payload.len() > MAXIMUM_PIN_PAYLOAD_BYTES {
            return Err(RootPinDenial::PayloadTooLarge);
        }
        let registry = self.serving.root_protection();
        let pins = registry.pins();
        let _journal = pins.lock_journal();
        let (pin, root) = self
            .serving
            .pin_current_root(parent, policy, payload.clone())?;
        let event = RootPinEvent::Acquire {
            pin,
            root: root.generation(),
            policy,
            parent,
            inline_tail: crate::physical_runtime::stability::inline_tail_of(&root),
            payload,
        };
        match self.append(&event, write) {
            Ok(()) => Ok(PhysicalRootPinReceipt {
                pin,
                root_generation: root.generation(),
            }),
            Err(AppendFate::NoEffect) => {
                pins.withdraw(pin);
                Err(RootPinDenial::StorageUnavailable)
            }
            Err(AppendFate::Indeterminate) => Err(RootPinDenial::StorageIndeterminate),
        }
    }

    /// Admit one in-process reader loan; `Expiring` and `Expired` pins refuse.
    pub fn loan(&self, pin: RootPinId) -> Result<PhysicalRootPinLoan<'runtime>, RootPinDenial> {
        PhysicalRootPinLoan::admit(self.serving, pin)
    }

    /// Request expiry: `Active -> Expiring` now, then complete
    /// `Expiring -> Expired` for this pin and each ancestor once drained.
    pub fn expire(
        &self,
        pin: RootPinId,
        write: PhysicalRootPinWrite,
    ) -> Result<RootPinState, RootPinDenial> {
        let registry = self.serving.root_protection();
        let pins = registry.pins();
        let _journal = pins.lock_journal();
        if pins.expiry(pin)? == RootPinExpiry::BeginExpiring {
            match self.append(&RootPinEvent::Expiring { pin }, write) {
                Ok(()) => {
                    pins.expire(pin)?;
                }
                Err(AppendFate::Indeterminate) => {
                    let _ = pins.expire(pin);
                    return Err(RootPinDenial::StorageIndeterminate);
                }
                Err(AppendFate::NoEffect) => {
                    return Err(RootPinDenial::StorageUnavailable);
                }
            }
        }
        self.settle(pin, write)?;
        pins.state(pin).ok_or(RootPinDenial::PinNotFound)
    }

    /// Maintenance step: complete every drained `Expiring` pin and its drained
    /// `Expiring` ancestors. Returns how many pins reached `Expired`.
    pub fn settle_drained(&self, write: PhysicalRootPinWrite) -> Result<usize, RootPinDenial> {
        let registry = self.serving.root_protection();
        let pins = registry.pins();
        let _journal = pins.lock_journal();
        let before = pins.live_pins();
        for pin in pins.drained()? {
            self.settle(pin, write)?;
        }
        Ok(before.saturating_sub(pins.live_pins()))
    }

    pub fn state(&self, pin: RootPinId) -> Option<RootPinState> {
        self.serving.root_protection().pins().state(pin)
    }

    pub fn root_generation(&self, pin: RootPinId) -> Option<u64> {
        self.serving.root_protection().pins().root(pin)
    }

    /// The owner bytes recorded with a pin that still protects its root.
    pub fn payload(&self, pin: RootPinId) -> Option<Vec<u8>> {
        self.serving.root_protection().pins().payload(pin)
    }

    /// The head position `journal` held in the pinned root itself, so every
    /// component coordinate names exactly the pinned frontier.
    pub fn pinned_journal_head(&self, pin: RootPinId, journal: OrderedJournalName) -> Option<u64> {
        self.serving
            .root_protection()
            .pins()
            .journal_head(pin, journal.anchor())
    }

    /// Pins that still protect a root.
    pub fn live_pins(&self) -> usize {
        self.serving.root_protection().pins().live_pins()
    }

    /// Whether open replayed the pin journal. An unrestored table protects
    /// every root and admits no pin operation.
    pub fn restored(&self) -> bool {
        self.serving.root_protection().pins().restored()
    }

    fn settle(&self, pin: RootPinId, write: PhysicalRootPinWrite) -> Result<(), RootPinDenial> {
        let registry = self.serving.root_protection();
        let pins = registry.pins();
        let mut next = Some(pin);
        while let Some(current) = next {
            if pins.expiry(current)? != RootPinExpiry::Complete {
                return Ok(());
            }
            self.append(&RootPinEvent::Expired { pin: current }, write)
                .map_err(storage)?;
            pins.expire(current)?;
            next = pins.parent(current);
        }
        Ok(())
    }

    fn append(&self, event: &RootPinEvent, write: PhysicalRootPinWrite) -> Result<(), AppendFate> {
        let journal = self
            .serving
            .ordered_journal(OrderedJournalName::physical_root_pins())
            .map_err(|_| AppendFate::NoEffect)?;
        let head = journal
            .head()
            .map_err(|_| AppendFate::NoEffect)?
            .map_or(0, |entry| entry.position());
        let position = head.checked_add(1).ok_or(AppendFate::NoEffect)?;
        journal
            .publish(OrderedJournalPublication::new(
                position,
                event.encode(),
                write.placement,
                write.deadline,
            ))
            .map(|_| ())
            .map_err(classify)
    }
}

fn classify(failure: OrderedJournalPublicationFailure) -> AppendFate {
    use OrderedJournalPublicationFailure as Failure;
    match failure {
        Failure::EmptyPayload
        | Failure::PositionGap { .. }
        | Failure::PositionConflict(_)
        | Failure::AnchorTableFull
        | Failure::Unresolved { .. }
        | Failure::Batch(_)
        | Failure::Idempotency(_)
        | Failure::Preparation(_)
        | Failure::ProvenNoEffect(_)
        | Failure::Read(_) => AppendFate::NoEffect,
        _ => AppendFate::Indeterminate,
    }
}

const fn storage(fate: AppendFate) -> RootPinDenial {
    match fate {
        AppendFate::NoEffect => RootPinDenial::StorageUnavailable,
        AppendFate::Indeterminate => RootPinDenial::StorageIndeterminate,
    }
}
