//! Continue the installed successor checkpoint without repeating publication.
macro_rules! rejoin_successor {
    ($label:lifetime; $phase:ident; $runtime:ident, $principal:ident, $request_scope:ident;
     $wave:ident, $resolved_on_wave:ident, $queue:ident, $frame_custody:ident;
     $demand:ident, $admission:ident; $current:ident, $current_contacts:ident, $current_role:ident, $successor_role:ident;
     $performed:ident; $hold_queue_frame:ident, $finish_caller:ident, $stopped:ident) => {{
                loop {
                    // Rejoin the actual successor after each real
                    // Published/Delivered stage. A deferred stage leaves its
                    // checkpoint installed and returns Pending without
                    // spinning or reexecuting.
                    $admission
                        .charge_external_work(2)
                        .map_err(|_| work_denial())?;
                    let custody = if $queue.active() {
                        &mut *$frame_custody
                    } else {
                        &mut $demand.required_continuations
                    };
                    let successor = custody
                        .last()
                        .expect("the prepared slot installed one successor");
                    if let Some(ready) = $runtime
                        .output_demands
                        .interest_ready_readmission(successor.interest(), $admission)?
                    {
                        $runtime
                            .output_demands
                            .clear_required_stop(successor.interest().key());
                        $current_contacts = if $wave.target == RequiredWaveTarget::Caller
                            && $successor_role == FrameRole::CallerSuccessor
                        {
                            $demand.producer_contacts_in_this_demand
                        } else {
                            successor.producer_contacts()
                        };
                        if committed_ready(&ready, $admission)? {
                            $wave = reselect_required_wave($runtime, $wave, $admission)?;
                            $resolved_on_wave.clear();
                            $queue.wave_moved();
                        }
                        if let Err(stop) = $performed.performed(&ready) {
                            $stopped!($label, ready.key(), stop);
                        }
                        $current = Some(ready);
                        $current_role = $successor_role;
                        continue $label;
                    }
                    $admission
                        .charge_external_work(1)
                        .map_err(|_| work_denial())?;
                    let successor = custody
                        .last_mut()
                        .expect("the prepared slot installed one successor");
                    let progressed = match successor.advance_checkpoint(
                        $phase,
                        $runtime,
                        $request_scope,
                        $admission,
                    ) {
                        Ok(progressed) => progressed,
                        Err(stop) => $stopped!($label, successor.interest().key(), stop),
                    };
                    if !progressed {
                        if $queue.active() {
                            $hold_queue_frame!($label, None)
                        }
                        $finish_caller!($label, WorthQueryOutputDemandAdvance::Pending)
                    }
                }
    }};
}
