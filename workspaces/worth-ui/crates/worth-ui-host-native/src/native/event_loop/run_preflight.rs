use std::cell::RefCell;
use std::rc::Rc;

use super::UiNativeEventLoopRunDenial;
use crate::native::readiness::UiNativeWakeSender;
use crate::native::{
    UiNativeHostState, UiNativeReadinessRegistry, UiNativeReadyOwner, UiNativeResourceClass,
    UiNativeResourceOwner,
};

pub(super) struct UiNativeEventLoopRunPreflight {
    pub readiness: UiNativeReadinessRegistry,
    pub readiness_owner: UiNativeReadyOwner,
    pub physical_readiness_owner: UiNativeReadyOwner,
    pub input_readiness_owner: UiNativeReadyOwner,
    pub application_readiness_owners: Box<[UiNativeReadyOwner]>,
    pub application_readiness_ports: Box<[crate::UiNativeApplicationReadinessPort]>,
    pub loop_resources: Vec<UiNativeResourceOwner>,
}

pub(super) fn prepare(
    state: &Rc<RefCell<UiNativeHostState>>,
    application_owner_count: crate::UiNativeApplicationReadinessOwnerCount,
    wake: &UiNativeWakeSender,
) -> Result<UiNativeEventLoopRunPreflight, UiNativeEventLoopRunDenial> {
    let loop_resources = state
        .borrow_mut()
        .resources
        .reserve(&[
            UiNativeResourceClass::ApplicationDriver,
            UiNativeResourceClass::EventWakeRegistration,
        ])
        .map_err(|_| UiNativeEventLoopRunDenial::ApplicationDriver)?;
    let readiness = UiNativeReadinessRegistry::new();
    let readiness_owner = match readiness.register() {
        Ok(owner) => owner,
        Err(()) => {
            release_resources(state, loop_resources);
            return Err(UiNativeEventLoopRunDenial::ApplicationDriver);
        }
    };
    let physical_readiness_owner = match readiness.register_level() {
        Ok(owner) => owner,
        Err(()) => {
            readiness.close();
            release_resources(state, loop_resources);
            return Err(UiNativeEventLoopRunDenial::IncompleteCleanup);
        }
    };
    let input_readiness_owner = match readiness.register_level() {
        Ok(owner) => owner,
        Err(()) => {
            readiness.close();
            release_resources(state, loop_resources);
            return Err(UiNativeEventLoopRunDenial::IncompleteCleanup);
        }
    };
    let application_readiness_owners = (0..application_owner_count.get())
        .map(|_| readiness.register_level())
        .collect::<Result<Vec<_>, _>>();
    let application_readiness_owners = match application_readiness_owners {
        Ok(owners) => owners.into_boxed_slice(),
        Err(()) => {
            readiness.close();
            release_resources(state, loop_resources);
            return Err(UiNativeEventLoopRunDenial::ApplicationDriver);
        }
    };
    let application_readiness_ports = application_readiness_owners
        .iter()
        .copied()
        .map(|owner| {
            crate::UiNativeApplicationReadinessPort::new(readiness.clone(), owner, wake.clone())
        })
        .collect::<Vec<_>>()
        .into_boxed_slice();
    Ok(UiNativeEventLoopRunPreflight {
        readiness,
        readiness_owner,
        physical_readiness_owner,
        input_readiness_owner,
        application_readiness_owners,
        application_readiness_ports,
        loop_resources,
    })
}

pub(super) fn cancel(
    state: &Rc<RefCell<UiNativeHostState>>,
    readiness: &UiNativeReadinessRegistry,
    expected: &[UiNativeReadyOwner],
    loop_resources: Vec<UiNativeResourceOwner>,
) {
    let _ = readiness.close_exact(expected);
    release_resources(state, loop_resources);
}

fn release_resources(
    state: &Rc<RefCell<UiNativeHostState>>,
    resources: Vec<UiNativeResourceOwner>,
) {
    state
        .borrow_mut()
        .resources
        .release_all(resources)
        .expect("event-loop preflight owners remain exact");
}
