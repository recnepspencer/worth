//! One canonical root deriving every sub-identity, with explicit overrides.
use super::grammar::{
    extends, is_dotted_identity, is_segment, same_text, MAXIMUM_APPLICATION_IDENTITY_BYTES,
};
use super::role::ROLE_COUNT;
use super::{
    ApplicationDerivedIdentity, ApplicationIdentityNamespaceDenial, ApplicationIdentityRole,
};

const SLOT_SEGMENT: &str = "slot";

/// The validated root of an action, query or feature's identities.
///
/// `identity(role)` is `root.<segment>` unless the role carries an explicit
/// override; `slot(field)` is `root.slot.<field>`. Overrides keep persisted
/// wire identities, and may not reuse the root, another role's identity or
/// the slot space.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ApplicationIdentityNamespace {
    root: &'static str,
    overrides: [Option<&'static str>; ROLE_COUNT],
}

macro_rules! role_identities {
    ($($method:ident => $Role:ident),* $(,)?) => {
        $(
            #[doc = concat!("The `", stringify!($method), "` identity, or its explicit override.")]
            pub const fn $method(&self) -> ApplicationDerivedIdentity {
                self.identity(ApplicationIdentityRole::$Role)
            }
        )*
    };
}

impl ApplicationIdentityNamespace {
    /// Admits a root whose every role derivation fits the identity bound.
    pub const fn new(root: &'static str) -> Result<Self, ApplicationIdentityNamespaceDenial> {
        if !is_dotted_identity(root) {
            return Err(ApplicationIdentityNamespaceDenial::InvalidRoot);
        }
        let required = root.len() + 1 + ApplicationIdentityRole::longest_segment();
        if required > MAXIMUM_APPLICATION_IDENTITY_BYTES {
            return Err(overlong(required));
        }
        Ok(Self {
            root,
            overrides: [None; ROLE_COUNT],
        })
    }

    pub const fn root(&self) -> &'static str {
        self.root
    }

    /// Keeps an existing wire identity for one role instead of deriving it.
    pub const fn with_override(
        mut self,
        role: ApplicationIdentityRole,
        identity: &'static str,
    ) -> Result<Self, ApplicationIdentityNamespaceDenial> {
        if self.overrides[role.index()].is_some() {
            return Err(ApplicationIdentityNamespaceDenial::DuplicateOverride { role });
        }
        if !is_dotted_identity(identity) {
            return Err(ApplicationIdentityNamespaceDenial::InvalidOverride { role });
        }
        if identity.len() > MAXIMUM_APPLICATION_IDENTITY_BYTES {
            return Err(overlong(identity.len()));
        }
        if self.collides(role, identity) {
            return Err(ApplicationIdentityNamespaceDenial::OverrideCollision { role });
        }
        self.overrides[role.index()] = Some(identity);
        Ok(self)
    }

    /// The role's identity: its explicit override, else `root.<segment>`.
    pub const fn identity(&self, role: ApplicationIdentityRole) -> ApplicationDerivedIdentity {
        match self.overrides[role.index()] {
            Some(identity) => ApplicationDerivedIdentity::joined(&[identity]),
            None => ApplicationDerivedIdentity::joined(&[self.root, role.segment()]),
        }
    }

    role_identities!(
        input => Input,
        request => Request,
        parameters => Parameters,
        binding => Binding,
        query => Query,
        handler => Handler,
        idempotency => Idempotency,
        denial => Denial,
        result => Result,
        body => Body,
        completion => Completion,
        projection => Projection,
    );

    /// The identity of one result or capability slot: `root.slot.<field>`.
    pub const fn slot(
        &self,
        field: &str,
    ) -> Result<ApplicationDerivedIdentity, ApplicationIdentityNamespaceDenial> {
        if !is_segment(field) {
            return Err(ApplicationIdentityNamespaceDenial::InvalidSlotField);
        }
        let required = self.root.len() + 1 + SLOT_SEGMENT.len() + 1 + field.len();
        if required > MAXIMUM_APPLICATION_IDENTITY_BYTES {
            return Err(overlong(required));
        }
        Ok(ApplicationDerivedIdentity::joined(&[
            self.root,
            SLOT_SEGMENT,
            field,
        ]))
    }

    /// Whether `identity` would make two of this namespace's identities equal.
    const fn collides(&self, role: ApplicationIdentityRole, identity: &str) -> bool {
        if same_text(identity, self.root) || extends(identity, self.root, "slot.") {
            return true;
        }
        let mut index = 0;
        while index < ROLE_COUNT {
            let other = ApplicationIdentityRole::ALL[index];
            if other.index() != role.index() && same_text(identity, self.identity(other).as_str()) {
                return true;
            }
            index += 1;
        }
        false
    }
}

const fn overlong(required: usize) -> ApplicationIdentityNamespaceDenial {
    ApplicationIdentityNamespaceDenial::Overlong {
        required,
        maximum: MAXIMUM_APPLICATION_IDENTITY_BYTES,
    }
}
