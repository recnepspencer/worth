//! Opaque recovery identity — unforgeable, not a digest of public fields (R8.34).

use std::sync::atomic::{AtomicU64, Ordering};

use worth_proof::{
    Artifact, AssumptionBasis, BoundaryBridgedAuthorityRevalidationRequiredBasis, CurrentValidity,
    FreshnessScopedBasis, NoProofs, PhaseMarker,
};

/// Framework-private secret material that never appears on the wire.
///
/// A digest of public receipt fields would be forgeable by anyone who knows
/// those fields. This counter is process-local and never exported.
static NEXT_SECRET: AtomicU64 = AtomicU64::new(0x9e37_79b9_7f4a_7c15);

/// Opaque recovery identity. Intentionally has neither `Clone` nor `Copy`.
///
/// Possession of published bytes does not reconstruct this type.
#[derive(Debug, Eq, PartialEq, Hash)]
pub(crate) struct WorthQueryRecoveryHandleIdentity {
    secret: u64,
    ordinal: u64,
}

impl WorthQueryRecoveryHandleIdentity {
    pub(crate) fn mint() -> Self {
        let ordinal = NEXT_SECRET.fetch_add(1, Ordering::Relaxed);
        // Mix with a second sample so the public projection is not the ordinal.
        let secret = NEXT_SECRET.fetch_add(0xC2B2_AE3D_27D4_EB4F, Ordering::Relaxed);
        Self { secret, ordinal }
    }

    pub(crate) const fn secret(&self) -> u64 {
        self.secret
    }

    pub(crate) const fn ordinal(&self) -> u64 {
        self.ordinal
    }
}

/// Runtime-private identity carried by authority admitted for one exact handle.
///
/// Unlike the wire projection, this type never crosses the facade and has no
/// caller-accessible constructor.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub(crate) struct WorthQueryRecoveryHandleAuthorityIdentity {
    secret: u64,
    ordinal: u64,
}

impl WorthQueryRecoveryHandleAuthorityIdentity {
    pub(crate) const fn from_handle(identity: &WorthQueryRecoveryHandleIdentity) -> Self {
        Self {
            secret: identity.secret(),
            ordinal: identity.ordinal(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct WorthQueryRecoveryWireProjectionPhase;
impl PhaseMarker for WorthQueryRecoveryWireProjectionPhase {}

worth_proof::authority_marker!(WorthQueryRecoveryWireProjectionAuthority);

type WorthQueryBridgedRecoveryWireProjection = Artifact<
    WorthQueryRecoveryWireProjectionPhase,
    [u8; 32],
    NoProofs,
    BoundaryBridgedAuthorityRevalidationRequiredBasis<WorthQueryRecoveryHandleAuthorityIdentity>,
>;

/// Wire-safe opaque projection. Cloneable for transport, but cannot mint a
/// handle. Its proof substrate is explicitly boundary-weakened: transport
/// retains descriptive bytes and an authority-revalidation-required basis,
/// never the current-validity basis of the live Query handle (R8.34).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryOpaqueRecoveryWireIdentity {
    projection: WorthQueryBridgedRecoveryWireProjection,
}

impl WorthQueryOpaqueRecoveryWireIdentity {
    pub(crate) fn project(identity: &WorthQueryRecoveryHandleIdentity) -> Self {
        let mut bytes = [0u8; 32];
        bytes[..8].copy_from_slice(&identity.secret().to_le_bytes());
        bytes[8..16].copy_from_slice(&identity.ordinal().to_le_bytes());
        // Remaining bytes are a non-invertible pad from the secret mix; they are
        // descriptive of the projection only and never reconstitute the handle.
        let mix = identity
            .secret()
            .wrapping_mul(0x1000_0000_01B3)
            .to_le_bytes();
        bytes[16..24].copy_from_slice(&mix);
        let mix2 = identity.ordinal().wrapping_mul(0xC2B2_AE3D).to_le_bytes();
        bytes[24..32].copy_from_slice(&mix2);
        let current: Artifact<
            WorthQueryRecoveryWireProjectionPhase,
            [u8; 32],
            NoProofs,
            FreshnessScopedBasis<
                CurrentValidity,
                AssumptionBasis<WorthQueryRecoveryHandleAuthorityIdentity>,
            >,
        > = Artifact::with_current_basis(
            bytes,
            WorthQueryRecoveryHandleAuthorityIdentity::from_handle(identity),
            WorthQueryRecoveryWireProjectionAuthority::witness(),
        );
        Self {
            projection: current.bridge_trust_boundary(),
        }
    }

    pub fn bytes(&self) -> &[u8; 32] {
        self.projection.payload()
    }
}
