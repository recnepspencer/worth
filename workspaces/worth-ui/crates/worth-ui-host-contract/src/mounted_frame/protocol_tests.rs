use super::{
    UiHostMeasurementSchemaVersion, UiHostObservationSchemaVersion, UiHostProtocolContract,
    UiHostProtocolDenial, UiHostProtocolIdentity, UiHostProtocolNegotiation,
    UiHostProtocolSchemaFamily, UiHostProtocolVersion, UiHostSolicitedEffectSchemaVersion,
    UiMountedFrameSchemaVersion, UiMountedPresentationSchemaVersion,
};

#[test]
fn protocol_and_each_schema_family_negotiate_without_cross_family_substitution() {
    let current = UiHostProtocolContract::current();
    let revisions = Revisions::current(current);
    assert!(matches!(
        current.negotiate(),
        UiHostProtocolNegotiation::Compatible(_)
    ));
    assert_denial(
        revisions.with_protocol(revisions.protocol - 1).contract(),
        UiHostProtocolDenial::ProtocolTooOld,
    );
    assert_denial(
        revisions.with_protocol(0).contract(),
        UiHostProtocolDenial::ProtocolTooOld,
    );
    assert_denial(
        revisions.with_protocol(revisions.protocol + 1).contract(),
        UiHostProtocolDenial::ProtocolTooNew,
    );

    for family in FAMILIES {
        assert_denial(
            revisions.with_schema(family, -1).contract(),
            UiHostProtocolDenial::SchemaTooOld(family),
        );
        assert_denial(
            revisions.with_schema(family, 1).contract(),
            UiHostProtocolDenial::SchemaTooNew(family),
        );
    }

    let foreign = revisions.contract_with_identity(UiHostProtocolIdentity::from_untrusted(7));
    assert_denial(foreign, UiHostProtocolDenial::ForeignIdentity);
}

const FAMILIES: [UiHostProtocolSchemaFamily; 5] = [
    UiHostProtocolSchemaFamily::MountedFrame,
    UiHostProtocolSchemaFamily::MountedPresentation,
    UiHostProtocolSchemaFamily::Observation,
    UiHostProtocolSchemaFamily::Measurement,
    UiHostProtocolSchemaFamily::SolicitedEffect,
];

#[test]
fn peers_without_surface_gradient_paint_are_denied() {
    let mut previous = Revisions::current(UiHostProtocolContract::current());
    previous.protocol = 8;
    previous.frame = 7;
    previous.presentation = 7;
    assert_denial(previous.contract(), UiHostProtocolDenial::ProtocolTooOld);
    previous.protocol = 9;
    assert_denial(
        previous.contract(),
        UiHostProtocolDenial::SchemaTooOld(UiHostProtocolSchemaFamily::MountedFrame),
    );
    previous.frame = 8;
    assert_denial(
        previous.contract(),
        UiHostProtocolDenial::SchemaTooOld(UiHostProtocolSchemaFamily::MountedPresentation),
    );
}

#[derive(Clone, Copy)]
struct Revisions {
    protocol: u16,
    frame: u16,
    presentation: u16,
    observation: u16,
    measurement: u16,
    solicited: u16,
}

impl Revisions {
    fn current(contract: UiHostProtocolContract) -> Self {
        Self {
            protocol: contract.protocol().revision(),
            frame: contract.mounted_frame().revision(),
            presentation: contract.mounted_presentation().revision(),
            observation: contract.observation().revision(),
            measurement: contract.measurement().revision(),
            solicited: contract.solicited_effect().revision(),
        }
    }

    fn with_protocol(mut self, revision: u16) -> Self {
        self.protocol = revision;
        self
    }

    fn with_schema(mut self, family: UiHostProtocolSchemaFamily, delta: i16) -> Self {
        let revision = match family {
            UiHostProtocolSchemaFamily::MountedFrame => &mut self.frame,
            UiHostProtocolSchemaFamily::MountedPresentation => &mut self.presentation,
            UiHostProtocolSchemaFamily::Observation => &mut self.observation,
            UiHostProtocolSchemaFamily::Measurement => &mut self.measurement,
            UiHostProtocolSchemaFamily::SolicitedEffect => &mut self.solicited,
        };
        *revision = revision.saturating_add_signed(delta);
        self
    }

    fn contract(self) -> UiHostProtocolContract {
        self.contract_with_identity(UiHostProtocolIdentity::worth_ui())
    }

    fn contract_with_identity(self, identity: UiHostProtocolIdentity) -> UiHostProtocolContract {
        UiHostProtocolContract::new(
            identity,
            UiHostProtocolVersion::new(self.protocol),
            UiMountedFrameSchemaVersion::new(self.frame),
            UiMountedPresentationSchemaVersion::new(self.presentation),
            UiHostObservationSchemaVersion::new(self.observation),
            UiHostMeasurementSchemaVersion::new(self.measurement),
            UiHostSolicitedEffectSchemaVersion::new(self.solicited),
        )
    }
}

fn assert_denial(contract: UiHostProtocolContract, expected: UiHostProtocolDenial) {
    assert_eq!(
        contract.negotiate(),
        UiHostProtocolNegotiation::Incompatible(expected)
    );
}
