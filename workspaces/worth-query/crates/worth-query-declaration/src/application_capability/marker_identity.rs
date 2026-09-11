//! Declaration-owned identities for application capability marker families.

use crate::application_schema::ApplicationEntityMarkerIdentity;
use crate::portable_identity::WorthQueryPortableType;

pub trait ApplicationCapabilityMarkerIdentity: WorthQueryPortableType {
    type Schema;
    const IDENTIFIER: &'static str;
}

pub trait ApplicationCapabilityContextMarkerIdentity: WorthQueryPortableType {
    type Schema;
    const IDENTIFIER: &'static str;
}

pub trait ApplicationCapabilityProvenanceMarkerIdentity: WorthQueryPortableType {
    type Schema;
    const IDENTIFIER: &'static str;
}

pub trait ApplicationCapabilityContextEntitySlotMarkerIdentity: WorthQueryPortableType {
    type Schema;
    type Context: ApplicationCapabilityContextMarkerIdentity<Schema = Self::Schema>;
    type Entity: ApplicationEntityMarkerIdentity<Self::Schema>;
    const IDENTIFIER: &'static str;
}
