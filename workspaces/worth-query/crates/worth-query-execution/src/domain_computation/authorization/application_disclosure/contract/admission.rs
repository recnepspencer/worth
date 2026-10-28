//! Work facts emitted by the one compiler immediately before owned operations.
use super::WorthQueryApplicationDisclosureContractDenial;

pub(in crate::domain_computation::authorization::application_disclosure) enum CompilationWork<'a> {
    Visit,
    Copy(usize),
    Scan {
        entries: usize,
        parts: &'a [&'a str],
    },
    Lookup {
        entries: usize,
        parts: &'a [&'a str],
    },
}

pub(in crate::domain_computation::authorization::application_disclosure) enum CompilationStop<E> {
    Admission(E),
    Contract(WorthQueryApplicationDisclosureContractDenial),
}

impl<E> From<WorthQueryApplicationDisclosureContractDenial> for CompilationStop<E> {
    fn from(cause: WorthQueryApplicationDisclosureContractDenial) -> Self {
        Self::Contract(cause)
    }
}

pub(in crate::domain_computation::authorization::application_disclosure) fn pay<E>(
    admit: &mut impl FnMut(CompilationWork<'_>) -> Result<(), E>,
    work: CompilationWork<'_>,
) -> Result<(), CompilationStop<E>> {
    admit(work).map_err(CompilationStop::Admission)
}

pub(super) fn unmetered<T>(
    result: Result<T, CompilationStop<std::convert::Infallible>>,
) -> Result<T, WorthQueryApplicationDisclosureContractDenial> {
    result.map_err(|cause| match cause {
        CompilationStop::Admission(never) => match never {},
        CompilationStop::Contract(cause) => cause,
    })
}
