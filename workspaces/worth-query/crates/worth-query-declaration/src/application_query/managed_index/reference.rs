//! Typed associations resolve genuine source and probe owners at installation.
use std::marker::PhantomData;

use crate::application_query::ApplicationQueryBinding;
use crate::application_schema::{
    ApplicationEntityMarkerIdentity, ApplicationReadableScalarValueBinding, ApplicationSchema,
};

/// One application-owned provider contract. This declares no live root or read authority.
pub trait ApplicationManagedIndexBinding<Schema: ApplicationSchema>: Sized + 'static {
    type SourceBinding: ApplicationQueryBinding<Schema>;
    type ProbeBinding: ApplicationReadableScalarValueBinding;
    type Entity: ApplicationEntityMarkerIdentity<Schema>;

    const NAME: &'static str;
    const PROVIDER: &'static str;
    const PROVIDER_VERSION: u32;

    #[doc(hidden)]
    fn descriptor() -> super::ApplicationManagedIndexBindingDescriptor {
        super::ApplicationManagedIndexBindingDescriptor::of::<Schema, Self>()
    }

    fn reference() -> ApplicationManagedIndexRef<Schema, Self> {
        ApplicationManagedIndexRef {
            name: Self::NAME,
            _marker: PhantomData,
        }
    }
}

/// A descriptive typed selector. Ordinary reads still require installed session admission.
pub struct ApplicationManagedIndexRef<Schema, Index> {
    name: &'static str,
    _marker: PhantomData<fn() -> (Schema, Index)>,
}

impl<S, I> ApplicationManagedIndexRef<S, I> {
    pub const fn name(&self) -> &'static str {
        self.name
    }
}
impl<S, I> Copy for ApplicationManagedIndexRef<S, I> {}
impl<S, I> Clone for ApplicationManagedIndexRef<S, I> {
    fn clone(&self) -> Self {
        *self
    }
}
