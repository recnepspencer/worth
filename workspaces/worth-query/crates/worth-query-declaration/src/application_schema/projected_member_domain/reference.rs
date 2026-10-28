//! Typed descriptive selectors do not issue physical identities or read proofs.
use crate::application_schema::ApplicationEntityRef;
use std::marker::PhantomData;

pub struct ApplicationProjectedMemberDomainRef<Schema, Domain, Carrier> {
    name: &'static str,
    carrier: &'static str,
    marker: PhantomData<fn() -> (Schema, Domain, Carrier)>,
}
impl<S, D, C> Copy for ApplicationProjectedMemberDomainRef<S, D, C> {}
impl<S, D, C> Clone for ApplicationProjectedMemberDomainRef<S, D, C> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<S, D, C> ApplicationProjectedMemberDomainRef<S, D, C> {
    pub const fn new(name: &'static str, carrier: ApplicationEntityRef<S, C>) -> Self {
        Self {
            name,
            carrier: carrier.name(),
            marker: PhantomData,
        }
    }
    pub const fn name(self) -> &'static str {
        self.name
    }
    pub const fn carrier(self) -> &'static str {
        self.carrier
    }
}

pub struct ApplicationProjectedMemberCaseRef<Schema, Domain, Case> {
    domain: &'static str,
    name: &'static str,
    marker: PhantomData<fn() -> (Schema, Domain, Case)>,
}
impl<S, D, C> Copy for ApplicationProjectedMemberCaseRef<S, D, C> {}
impl<S, D, C> Clone for ApplicationProjectedMemberCaseRef<S, D, C> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<S, D, Case> ApplicationProjectedMemberCaseRef<S, D, Case> {
    pub const fn new<Carrier>(
        domain: ApplicationProjectedMemberDomainRef<S, D, Carrier>,
        name: &'static str,
    ) -> Self {
        Self {
            domain: domain.name(),
            name,
            marker: PhantomData,
        }
    }
    pub const fn domain(self) -> &'static str {
        self.domain
    }
    pub const fn name(self) -> &'static str {
        self.name
    }
}
