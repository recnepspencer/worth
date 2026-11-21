//! One live representation of each admitted durable material obligation.
use super::*;
use std::collections::BTreeMap;
use worth_foundational::facade::{
    CorrectionMaterialPinIdentity as Identity, CorrectionMaterialPinOwner,
    CorrectionMaterialPinTerminal as Terminal,
};
use worth_proof::{AuthorityWitness, HomeQueryCorrectionsAuthority};

type Pin = worth_proof::LinearResource<Identity, Terminal, HomeQueryCorrectionsAuthority>;
#[derive(Debug, Default)]
pub(super) struct LiveMaterialPins(BTreeMap<Identity, Pin>);
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum MaterialPinChange {
    None,
    Acquire(Identity),
    Transfer(Identity, CorrectionMaterialPinOwner),
    Release(Identity),
}
impl LiveMaterialPins {
    pub(super) fn contains(&self, identity: Identity) -> bool {
        self.0.contains_key(&identity)
    }

    pub(super) fn readmit(
        catalog: &MaterialCatalog,
        authority: &HomeQueryCorrectionsAuthority,
    ) -> Self {
        let witness = AuthorityWitness::from_authority_marker(authority.clone());
        Self(
            catalog
                .pins
                .keys()
                .map(|identity| (*identity, Pin::mint(*identity, &witness)))
                .collect(),
        )
    }
    pub(super) fn validate(&self, record: &MaterialRecord) -> Result<(), Denial> {
        match record {
            MaterialRecord::Pin { identity, .. } | MaterialRecord::RecoveryPin { identity, .. }
                if self.0.contains_key(identity) =>
            {
                Err(Denial::PinAlreadyClaimed)
            }
            MaterialRecord::Pin { .. } | MaterialRecord::RecoveryPin { .. } => Ok(()),
            MaterialRecord::Transfer { identity, .. }
            | MaterialRecord::Release { identity, .. }
            | MaterialRecord::DischargePreparation { identity, .. } => {
                if self.0.contains_key(identity) {
                    Ok(())
                } else {
                    Err(Denial::UnknownPin)
                }
            }
            MaterialRecord::Capture { .. }
            | MaterialRecord::Ready { .. }
            | MaterialRecord::Abandon { .. }
            | MaterialRecord::Expire { .. } => Ok(()),
        }
    }
    pub(super) fn committed(
        &mut self,
        change: MaterialPinChange,
        authority: &HomeQueryCorrectionsAuthority,
    ) {
        let witness = AuthorityWitness::from_authority_marker(authority.clone());
        match change {
            MaterialPinChange::None => {}
            MaterialPinChange::Acquire(identity) => {
                assert!(self
                    .0
                    .insert(identity, Pin::mint(identity, &witness))
                    .is_none());
            }
            MaterialPinChange::Transfer(identity, owner) => {
                let pin = self.0.remove(&identity).expect("prevalidated original pin");
                let _receipt = pin.terminate(Terminal::Transferred(owner));
                self.0.insert(identity, Pin::mint(identity, &witness));
            }
            MaterialPinChange::Release(identity) => {
                let _receipt = self
                    .0
                    .remove(&identity)
                    .expect("prevalidated original pin")
                    .terminate(Terminal::Released);
            }
        }
    }
}
impl MaterialPinChange {
    pub(super) fn from_record(record: &MaterialRecord, catalog: &MaterialCatalog) -> Self {
        match record {
            MaterialRecord::Pin { identity, .. } | MaterialRecord::RecoveryPin { identity, .. } => {
                Self::Acquire(*identity)
            }
            MaterialRecord::Transfer {
                identity, after, ..
            } => Self::Transfer(*identity, *after),
            MaterialRecord::Release { identity, .. } => Self::Release(*identity),
            MaterialRecord::DischargePreparation { identity, .. } => Self::Release(*identity),
            MaterialRecord::Abandon { original } => catalog
                .pins
                .iter()
                .find_map(|(identity, row)| {
                    (row.owner
                        == CorrectionMaterialPinOwner::Publication {
                            original: *original,
                        })
                    .then_some(Self::Release(*identity))
                })
                .unwrap_or(Self::None),
            MaterialRecord::Capture { .. }
            | MaterialRecord::Ready { .. }
            | MaterialRecord::Expire { .. } => Self::None,
        }
    }
}
impl Drop for LiveMaterialPins {
    fn drop(&mut self) {
        // Every entry was minted only after its durable row was acknowledged or
        // readmitted. Ending this live representation does not release that row,
        // prove a no-effect, or acknowledge pending publication. A future Home
        // must readmit the actual prefix before representing it under a new lease.
        for (_, pin) in std::mem::take(&mut self.0) {
            let _receipt = pin.terminate(Terminal::OwnerAbandoned);
        }
    }
}
