use super::{
    UiHostObservationFamily, UiHostObservationPayload, UiHostObservationSequence,
    UiHostObservationTimeBasis, UiHostPointerDeviceKind,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiHostObservationMountedBasis {
    instance: crate::UiMountedInstanceIdentity,
    node_receipt: crate::UiMountedNodeReceiptIdentity,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiHostObservationPointerDeviceKindDenial {
    NotPointerPayload(UiHostObservationFamily),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UiHostObservationReport {
    sequence: UiHostObservationSequence,
    time_basis: UiHostObservationTimeBasis,
    payload: UiHostObservationPayload,
    mounted_basis: Option<UiHostObservationMountedBasis>,
    input_affinity: Option<super::UiHostInputRecipientAffinityReceipt>,
    pointer_device_kind: Option<UiHostPointerDeviceKind>,
}

impl UiHostObservationMountedBasis {
    pub const fn new(
        instance: crate::UiMountedInstanceIdentity,
        node_receipt: crate::UiMountedNodeReceiptIdentity,
    ) -> Self {
        Self {
            instance,
            node_receipt,
        }
    }

    pub const fn instance(self) -> crate::UiMountedInstanceIdentity {
        self.instance
    }

    pub const fn node_receipt(self) -> crate::UiMountedNodeReceiptIdentity {
        self.node_receipt
    }
}

impl UiHostObservationReport {
    pub fn new(
        sequence: UiHostObservationSequence,
        time_basis: UiHostObservationTimeBasis,
        payload: UiHostObservationPayload,
    ) -> Self {
        Self {
            sequence,
            time_basis,
            payload,
            mounted_basis: None,
            input_affinity: None,
            pointer_device_kind: None,
        }
    }

    pub fn with_mounted_basis(mut self, basis: UiHostObservationMountedBasis) -> Self {
        self.mounted_basis = Some(basis);
        self
    }

    pub fn with_input_affinity(
        mut self,
        affinity: super::UiHostInputRecipientAffinityReceipt,
    ) -> Self {
        self.input_affinity = Some(affinity);
        self
    }

    pub fn with_pointer_device_kind(
        mut self,
        kind: UiHostPointerDeviceKind,
    ) -> Result<Self, UiHostObservationPointerDeviceKindDenial> {
        if !matches!(
            self.payload,
            UiHostObservationPayload::PointerMotion { .. }
                | UiHostObservationPayload::PointerButton { .. }
        ) {
            return Err(
                UiHostObservationPointerDeviceKindDenial::NotPointerPayload(self.family()),
            );
        }
        self.pointer_device_kind = Some(kind);
        Ok(self)
    }

    pub const fn sequence(&self) -> UiHostObservationSequence {
        self.sequence
    }

    pub const fn time_basis(&self) -> UiHostObservationTimeBasis {
        self.time_basis
    }

    pub fn payload(&self) -> &UiHostObservationPayload {
        &self.payload
    }

    pub const fn family(&self) -> UiHostObservationFamily {
        self.payload.family()
    }

    pub const fn mounted_basis(&self) -> Option<UiHostObservationMountedBasis> {
        self.mounted_basis
    }

    pub const fn input_affinity(&self) -> Option<super::UiHostInputRecipientAffinityReceipt> {
        self.input_affinity
    }

    pub const fn pointer_device_kind(&self) -> Option<UiHostPointerDeviceKind> {
        self.pointer_device_kind
    }

    pub const fn effective_pointer_device_kind(&self) -> Option<UiHostPointerDeviceKind> {
        match self.payload {
            UiHostObservationPayload::PointerMotion { .. }
            | UiHostObservationPayload::PointerButton { .. } => Some(
                match self.pointer_device_kind {
                    Some(kind) => kind,
                    None => UiHostPointerDeviceKind::Mouse,
                },
            ),
            _ => None,
        }
    }

    pub const fn coalescing_identity(
        &self,
    ) -> Option<super::UiHostObservationCoalescingIdentity> {
        match self.payload.coalescing_identity() {
            Some(super::UiHostObservationCoalescingIdentity::PointerMotion {
                pointer,
                capture_epoch,
                pressed_buttons,
                ..
            }) => Some(super::UiHostObservationCoalescingIdentity::PointerMotion {
                pointer,
                capture_epoch,
                pressed_buttons,
                device_kind: match self.pointer_device_kind {
                    Some(kind) => kind,
                    None => UiHostPointerDeviceKind::Mouse,
                },
            }),
            other => other,
        }
    }

    pub fn encoded_len(&self) -> usize {
        24 + self.payload.encoded_len()
            + usize::from(self.mounted_basis.is_some()) * 16
            + usize::from(self.input_affinity.is_some()) * 96
            + usize::from(self.pointer_device_kind.is_some())
    }

    pub fn input_affine_encoded_len(payload: &UiHostObservationPayload) -> usize {
        24 + payload.encoded_len() + 96
    }
}
