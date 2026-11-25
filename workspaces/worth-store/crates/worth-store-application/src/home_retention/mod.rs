//! The Home's whole-frontier retention owner (Store Milestone 4, Phase M4.1).
//!
//! [`DurableHomeRetention`] implements World's retention port over Store's
//! durable root pins: one constant-size pin entry per frontier, restored on
//! reopen before any retirement. The sealed [`HomeRootManifest`] is derived
//! from the pinned root's own journal anchors, so Native, World, Signal and the
//! product-map coordinates all name exactly the pinned frontier.
mod payload;
pub mod volatile;

use std::sync::Arc;
use worth_foundational::facade::HomeIncarnation;
use worth_runtime_world::facade::{
    FrontierPinCustody, HomeFrontierComponents, HomeRootManifest, LeaseState, LeaseTransition,
    NestedDropPolicy, PhysicalRootPointer, PinId, PinSpec, PinnedFrontierLease,
    RetentionPortDenial, RuntimeWorldRetentionPort,
};
use worth_store::physical_runtime::stability::durable_pins::{
    PhysicalRootPinWrite, PhysicalRootPins,
};
use worth_store::physical_runtime::{
    OrderedJournalName, PhysicalMutationDeadline, ServingPhysicalRuntime,
};
use worth_store::pin_table::{RootPinDenial, RootPinDropPolicy, RootPinId, RootPinState};

/// Upper bound for one pin entry's durable write.
const PIN_WRITE_MILLISECONDS: u64 = 30_000;

struct Inner {
    serving: Arc<ServingPhysicalRuntime>,
    incarnation: HomeIncarnation,
    program_digest: [u8; 32],
}

/// Durable whole-Home frontier pins of one serving Home.
#[derive(Clone)]
pub struct DurableHomeRetention {
    inner: Arc<Inner>,
}

impl std::fmt::Debug for DurableHomeRetention {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DurableHomeRetention")
            .field("incarnation", &self.inner.incarnation)
            .finish_non_exhaustive()
    }
}

impl DurableHomeRetention {
    pub(crate) fn new(
        serving: Arc<ServingPhysicalRuntime>,
        incarnation: HomeIncarnation,
        program_digest: [u8; 32],
    ) -> Self {
        Self {
            inner: Arc::new(Inner {
                serving,
                incarnation,
                program_digest,
            }),
        }
    }

    fn pins(&self) -> Result<PhysicalRootPins<'_>, RetentionPortDenial> {
        self.inner.serving.root_pins().map_err(denial)
    }

    fn write() -> Result<PhysicalRootPinWrite, RetentionPortDenial> {
        Ok(PhysicalRootPinWrite {
            placement: crate::open::physical_admission::placement(),
            deadline: PhysicalMutationDeadline::after_milliseconds(PIN_WRITE_MILLISECONDS)
                .ok_or(RetentionPortDenial::StorageUnavailable)?,
        })
    }

    /// Recompute the sealed frontier of a live pin from its pinned root.
    fn manifest_of(
        &self,
        pins: &PhysicalRootPins<'_>,
        pin: RootPinId,
    ) -> Result<HomeRootManifest, RetentionPortDenial> {
        let generation = pins
            .root_generation(pin)
            .ok_or(RetentionPortDenial::PinNotFound)?;
        let recorded = pins.payload(pin).ok_or(RetentionPortDenial::PinNotFound)?;
        let recorded = payload::decode(&recorded).ok_or(RetentionPortDenial::CorruptManifest)?;
        let head = |journal: &str| -> Result<PhysicalRootPointer, RetentionPortDenial> {
            let name = OrderedJournalName::new(journal)
                .map_err(|_| RetentionPortDenial::CorruptManifest)?;
            let position = pins
                .pinned_journal_head(pin, name)
                .ok_or(RetentionPortDenial::StorageUnavailable)?;
            Ok(PhysicalRootPointer::from_retention_owner(
                generation, position,
            ))
        };
        Ok(HomeRootManifest::seal(
            recorded.incarnation,
            generation,
            HomeFrontierComponents {
                native_root: head(worth_store::physical_runtime::RELATIONAL_CANONICAL_COMMITS)?,
                world_root: head("world.occurrences")?,
                signal_root: head("signal.advances")?,
                product_map_root: head("world.branches")?,
            },
            recorded.program_digest,
        ))
    }
}

impl RuntimeWorldRetentionPort for DurableHomeRetention {
    fn capture_and_pin_frontier(
        &self,
        spec: PinSpec,
    ) -> Result<PinnedFrontierLease, RetentionPortDenial> {
        let pins = self.pins()?;
        let recorded = payload::encode(
            self.inner.incarnation,
            self.inner.program_digest,
            spec.scope,
        );
        let receipt = pins
            .pin_current(
                spec.parent_pin.map(root_pin),
                policy(spec.nested_drop_policy),
                recorded,
                Self::write()?,
            )
            .map_err(denial)?;
        let manifest = self.manifest_of(&pins, receipt.pin())?;
        let pin_id = PinId::from_retention_owner(receipt.pin().ordinal());
        Ok(PinnedFrontierLease::from_retention_owner(
            pin_id,
            manifest,
            Box::new(DurableFrontierCustody {
                retention: self.clone(),
                pin: pin_id,
            }),
        ))
    }

    fn expire_lease(&self, pin: PinId) -> Result<LeaseTransition, RetentionPortDenial> {
        let state = self
            .pins()?
            .expire(root_pin(pin), Self::write()?)
            .map_err(denial)?;
        Ok(LeaseTransition::new(pin, lease_state(state)))
    }

    fn lease_state(&self, pin: PinId) -> Option<LeaseState> {
        self.pins().ok()?.state(root_pin(pin)).map(lease_state)
    }

    fn pinned_manifest(&self, pin: PinId) -> Result<HomeRootManifest, RetentionPortDenial> {
        self.manifest_of(&self.pins()?, root_pin(pin))
    }

    fn settle_drained(&self) -> Result<usize, RetentionPortDenial> {
        self.pins()?.settle_drained(Self::write()?).map_err(denial)
    }
}

#[derive(Debug)]
struct DurableFrontierCustody {
    retention: DurableHomeRetention,
    pin: PinId,
}

impl FrontierPinCustody for DurableFrontierCustody {
    fn read_pinned_manifest(&self) -> Result<HomeRootManifest, RetentionPortDenial> {
        let pins = self.retention.pins()?;
        let loan = pins.loan(root_pin(self.pin)).map_err(denial)?;
        let manifest = self.retention.manifest_of(&pins, loan.pin());
        drop(loan);
        manifest
    }

    fn release(&self) {
        // A refused or unresolved drop keeps the pin protecting its roots.
        let _ = self.retention.expire_lease(self.pin);
    }
}

pub(crate) const fn root_pin(pin: PinId) -> RootPinId {
    RootPinId::from_ordinal(pin.ordinal())
}

pub(crate) const fn policy(policy: NestedDropPolicy) -> RootPinDropPolicy {
    match policy {
        NestedDropPolicy::RefuseIfPinnedByChild => RootPinDropPolicy::RefuseIfPinnedByChild,
        NestedDropPolicy::RetainUntilChildrenDrain => RootPinDropPolicy::RetainUntilChildrenDrain,
    }
}

pub(crate) const fn lease_state(state: RootPinState) -> LeaseState {
    match state {
        RootPinState::Active => LeaseState::Active,
        RootPinState::Expiring => LeaseState::Expiring,
        RootPinState::Expired => LeaseState::Expired,
    }
}

pub(crate) const fn denial(denial: RootPinDenial) -> RetentionPortDenial {
    match denial {
        RootPinDenial::PinNotFound | RootPinDenial::DuplicatePin => {
            RetentionPortDenial::PinNotFound
        }
        RootPinDenial::ParentPinNotActive => RetentionPortDenial::ParentPinNotActive,
        RootPinDenial::LeaseExpired => RetentionPortDenial::LeaseExpired,
        RootPinDenial::PinnedByChild => RetentionPortDenial::PinnedByChild,
        RootPinDenial::StorageIndeterminate => RetentionPortDenial::StorageIndeterminate,
        RootPinDenial::CounterOverflow
        | RootPinDenial::PinIdentityExhausted
        | RootPinDenial::PayloadTooLarge
        | RootPinDenial::PinsUnavailable
        | RootPinDenial::ReclaimFenced
        | RootPinDenial::StorageUnavailable => RetentionPortDenial::StorageUnavailable,
    }
}
