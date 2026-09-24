use super::client_invocation::UiNativeEventLoopClientInvocation;
use super::{
    UiNativeEventLoopClient, UiNativeEventLoopDirective, UiNativeEventLoopRunDenial,
    UiNativeHostState,
};
use std::{cell::RefCell, rc::Rc};

pub(super) fn settle<Client: UiNativeEventLoopClient>(
    shared: &Rc<RefCell<UiNativeHostState>>,
    client: &mut Client,
    directive: UiNativeEventLoopDirective,
) -> Result<UiNativeEventLoopDirective, UiNativeEventLoopRunDenial> {
    if matches!(directive, UiNativeEventLoopDirective::Close) {
        return Ok(directive);
    }
    let grant = shared.borrow().lifecycle.begin_input_retention_recovery();
    let Some(grant) = grant else {
        return Ok(directive);
    };
    // The runtime can call the host adapter while cancelling interactions.
    // No host-state borrow may span this callback.
    let (acknowledgement, directive) = client
        .invoke_native_input_retention_exhausted(grant)
        .map_err(UiNativeEventLoopRunDenial::ClientCallback)?;
    if !shared
        .borrow_mut()
        .lifecycle
        .complete_input_retention_recovery(acknowledgement)
    {
        return Err(UiNativeEventLoopRunDenial::ApplicationDriver);
    }
    Ok(directive)
}

#[cfg(test)]
mod tests;
