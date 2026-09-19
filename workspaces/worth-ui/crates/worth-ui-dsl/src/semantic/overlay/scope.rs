#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct UiBackdropIdentity(u64);

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct UiPortalDeclarationId(u64);

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct UiSemanticSurfaceDeclarationIdentity(u64);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiBackdropScope {
    SurfaceSingleton,
    PerPortalInstance(UiPortalDeclarationId),
}

macro_rules! nonzero_identity {
    ($name:ident) => {
        impl $name {
            pub const fn new(value: u64) -> Option<Self> {
                if value == 0 {
                    None
                } else {
                    Some(Self(value))
                }
            }
            pub const fn value(self) -> u64 {
                self.0
            }
        }
    };
}

nonzero_identity!(UiBackdropIdentity);
nonzero_identity!(UiPortalDeclarationId);
nonzero_identity!(UiSemanticSurfaceDeclarationIdentity);
