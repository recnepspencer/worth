/// Actual decision-context access by a handler. This is an observed upper
/// bound check against a producer's static reuse declaration, not a source of
/// declared dependencies.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(in crate::domain_computation) struct DecisionContextUse(u8);

impl DecisionContextUse {
    const KEY: u8 = 1;
    const PRINCIPAL: u8 = 2;
    const SCOPE: u8 = 4;
    const OPAQUE_READER: u8 = 8;
    const MANAGED_COMPUTATION: u8 = 16;
    const REQUEST_CONTEXT: u8 = 32;

    pub(super) const fn key(self) -> Self {
        Self(self.0 | Self::KEY)
    }

    pub(super) const fn principal(self) -> Self {
        Self(self.0 | Self::PRINCIPAL)
    }

    pub(super) const fn scope(self) -> Self {
        Self(self.0 | Self::SCOPE)
    }

    pub(super) const fn opaque_reader(self) -> Self {
        Self(self.0 | Self::OPAQUE_READER)
    }

    pub(super) const fn managed_computation(self) -> Self {
        Self(self.0 | Self::MANAGED_COMPUTATION)
    }

    pub(super) const fn request_context(self) -> Self {
        Self(self.0 | Self::REQUEST_CONTEXT)
    }

    pub(in crate::domain_computation::primary_graph) const fn declared_context_bits(self) -> u8 {
        self.0 & (Self::KEY | Self::PRINCIPAL | Self::SCOPE)
    }

    pub(in crate::domain_computation::primary_graph) const fn has_untracked_context(self) -> bool {
        self.0 & (Self::OPAQUE_READER | Self::MANAGED_COMPUTATION | Self::REQUEST_CONTEXT) != 0
    }

    #[cfg(test)]
    pub(in crate::domain_computation::primary_graph) const fn raw_reader_for_test(self) -> Self {
        self.opaque_reader()
    }

    #[cfg(test)]
    pub(in crate::domain_computation::primary_graph) const fn key_for_test(self) -> Self {
        self.key()
    }
}
