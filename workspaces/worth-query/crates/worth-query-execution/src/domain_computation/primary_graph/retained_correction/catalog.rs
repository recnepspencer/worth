//! Semantic prefix and obligation fold; counts derive from durable pin rows.
use super::record::*;
use crate::domain_computation::application_aftermath::correction_material::{
    QueryCorrectionMaterialDenial as Denial, QueryCorrectionMaterialFrame,
};
use std::{collections::BTreeMap, sync::Arc};
use worth_foundational::facade::{
    CorrectionMaterialPinIdentity, CorrectionMaterialPinOwner,
    DurableCompositePublicationAttemptIdentity as Original, HomeIdentity,
};
#[cfg(test)]
mod tests;
mod validation;
#[derive(Clone, Debug)]
pub(in crate::domain_computation::primary_graph) struct MaterialMetadata {
    pub(in crate::domain_computation::primary_graph) program: [u8; 32],
    pub(in crate::domain_computation::primary_graph) mutation: String,
    pub(in crate::domain_computation::primary_graph) key: [u8; 32],
    pub(in crate::domain_computation::primary_graph) intent: String,
    pub(in crate::domain_computation::primary_graph) input_identity: [u8; 32],
    pub(in crate::domain_computation::primary_graph) intent_identity: [u8; 32],
}
#[derive(Clone, Debug)]
pub(in crate::domain_computation::primary_graph) enum MaterialBody {
    Captured(Arc<MaterialCapture>),
    Ready {
        capture: Arc<MaterialCapture>,
        ready: Arc<MaterialReady>,
    },
    Expired {
        commit: worth_relational::facade::history::RelationalCommitReceipt,
    },
    Abandoned,
}
#[derive(Clone, Debug)]
pub(in crate::domain_computation::primary_graph) struct MaterialEntry {
    pub(in crate::domain_computation::primary_graph) metadata: MaterialMetadata,
    pub(in crate::domain_computation::primary_graph) body: MaterialBody,
}
impl MaterialEntry {
    pub(in crate::domain_computation::primary_graph) fn available(
        &self,
    ) -> Result<(&Arc<MaterialCapture>, &Arc<MaterialReady>), Denial> {
        match &self.body {
            MaterialBody::Ready { capture, ready } => Ok((capture, ready)),
            MaterialBody::Captured(_) => Err(Denial::MaterialNotReady),
            MaterialBody::Expired { .. } => Err(Denial::RetainedMaterialExpired),
            MaterialBody::Abandoned => Err(Denial::Unavailable),
        }
    }
}
#[derive(Clone, Debug)]
pub(in crate::domain_computation::primary_graph) struct MaterialPinRow {
    pub(in crate::domain_computation::primary_graph) original: Original,
    pub(in crate::domain_computation::primary_graph) owner: CorrectionMaterialPinOwner,
    pub(in crate::domain_computation::primary_graph) prepared: Arc<PreparedCorrectionSnapshot>,
}
#[derive(Debug)]
pub(in crate::domain_computation::primary_graph) struct MaterialCatalog {
    pub(in crate::domain_computation::primary_graph) home: HomeIdentity,
    pub(in crate::domain_computation::primary_graph) position: u64,
    pub(in crate::domain_computation::primary_graph) digest: [u8; 32],
    pub(in crate::domain_computation::primary_graph) payload_digest: [u8; 32],
    pub(in crate::domain_computation::primary_graph) generation: u64,
    pub(in crate::domain_computation::primary_graph) material: BTreeMap<Original, MaterialEntry>,
    pub(in crate::domain_computation::primary_graph) pins:
        BTreeMap<CorrectionMaterialPinIdentity, MaterialPinRow>,
    pub(super) discharged: BTreeMap<[u8; 32], (Original, [u8; 32])>,
    pub(super) released: BTreeMap<CorrectionMaterialPinIdentity, CorrectionMaterialPinOwner>,
}
impl MaterialCatalog {
    pub(in crate::domain_computation::primary_graph) fn empty(home: HomeIdentity) -> Self {
        Self {
            home,
            position: 0,
            digest: [0; 32],
            payload_digest: [0; 32],
            generation: 0,
            material: BTreeMap::new(),
            pins: BTreeMap::new(),
            released: BTreeMap::new(),
            discharged: BTreeMap::new(),
        }
    }
    pub(in crate::domain_computation::primary_graph) fn frame(
        &self,
        generation: std::num::NonZeroU64,
        record: &MaterialRecord,
    ) -> Result<QueryCorrectionMaterialFrame, Denial> {
        let position = self
            .position
            .checked_add(1)
            .and_then(std::num::NonZeroU64::new)
            .ok_or(Denial::IdentityExhausted)?;
        self.validate(record, position.get())?;
        QueryCorrectionMaterialFrame::prepare(
            self.home,
            position,
            self.digest,
            generation,
            record.class(),
            &record.encode()?,
        )
    }
    pub(super) fn fold(
        &mut self,
        frame: &QueryCorrectionMaterialFrame,
    ) -> Result<super::pins::MaterialPinChange, Denial> {
        let record = MaterialRecord::decode(frame)?;
        if frame.home() != self.home {
            return Err(Denial::ForeignHome);
        }
        if self.position.checked_add(1) != Some(frame.position())
            || self.digest != frame.predecessor()
            || frame.publication_generation().get() <= self.generation
        {
            return Err(Denial::PrefixMismatch);
        }
        self.validate(&record, frame.position())?;
        let change = super::pins::MaterialPinChange::from_record(&record, self);
        match record {
            MaterialRecord::Capture { material } => {
                let capture: Arc<MaterialCapture> = Arc::from(material);
                self.material.insert(
                    capture.original,
                    MaterialEntry {
                        metadata: MaterialMetadata {
                            program: capture.program,
                            mutation: capture.mutation.clone(),
                            key: capture.key,
                            intent: capture.intent.clone(),
                            input_identity: capture.input_identity,
                            intent_identity: capture.intent_identity,
                        },
                        body: MaterialBody::Captured(capture),
                    },
                );
            }
            MaterialRecord::Ready { material } => {
                let entry = self
                    .material
                    .get_mut(&material.original)
                    .expect("validated original capture");
                let MaterialBody::Captured(capture) = &entry.body else {
                    unreachable!("validated capture phase")
                };
                entry.body = MaterialBody::Ready {
                    capture: Arc::clone(capture),
                    ready: Arc::from(material),
                };
            }
            MaterialRecord::Pin {
                identity,
                original,
                owner,
                prepared,
                ..
            } => {
                self.pins.insert(
                    identity,
                    MaterialPinRow {
                        original,
                        owner,
                        prepared: Arc::from(prepared),
                    },
                );
            }
            MaterialRecord::Transfer {
                identity, after, ..
            } => {
                self.pins
                    .get_mut(&identity)
                    .expect("validated current pin")
                    .owner = after;
            }
            MaterialRecord::Release {
                identity, owner, ..
            } => {
                self.pins.remove(&identity);
                self.released.insert(identity, owner);
            }
            MaterialRecord::DischargePreparation { identity, key, .. } => {
                let pin = self
                    .pins
                    .remove(&identity)
                    .expect("validated prepared command");
                self.discharged
                    .insert(key, (pin.original, pin.prepared.intent));
            }
            MaterialRecord::Abandon { original } => {
                if let super::pins::MaterialPinChange::Release(identity) = change {
                    let pin = self
                        .pins
                        .remove(&identity)
                        .expect("validated publication pin");
                    self.released.insert(identity, pin.owner);
                }
                self.material
                    .get_mut(&original)
                    .expect("validated capture")
                    .body = MaterialBody::Abandoned;
            }
            MaterialRecord::Expire { original } => {
                let entry = self
                    .material
                    .get_mut(&original)
                    .expect("validated original");
                let MaterialBody::Ready { ready, .. } = &entry.body else {
                    unreachable!("validated available material")
                };
                entry.body = MaterialBody::Expired {
                    commit: ready.commit.clone(),
                };
            }
        }
        self.position = frame.position();
        self.digest = frame.digest();
        self.payload_digest = frame.payload_digest();
        self.generation = frame.publication_generation().get();
        Ok(change)
    }
    pub(in crate::domain_computation::primary_graph) fn pin_count(
        &self,
        original: Original,
    ) -> usize {
        self.pins
            .values()
            .filter(|pin| pin.original == original || matches!(pin.owner,
                CorrectionMaterialPinOwner::Publication { original: publishing } if publishing == original))
            .count()
    }
}
