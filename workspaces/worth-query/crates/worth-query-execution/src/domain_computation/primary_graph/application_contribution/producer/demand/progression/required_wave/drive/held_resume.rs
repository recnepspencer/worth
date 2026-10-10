//! Resume the exact held row using its caller or queue custody.

// Both accepted dependencies and a fresh decision resume the exact held row.
macro_rules! resume_held {
    ($label:lifetime, $head:expr;
     $phase:ident;
     $runtime:ident, $principal:ident, $request_scope:ident;
     $wave:ident, $resolved_on_wave:ident, $queue:ident, $frame_custody:ident;
     $demand:ident, $admission:ident, $performed:ident;
     $hold_queue_frame:ident, $finish_caller:ident, $stopped:ident) => {{
        let head = $head;
        let custody = if $queue.active() || $wave.target == RequiredWaveTarget::Requested {
            ContinuationCustody::Queue(if $queue.active() { &mut *$frame_custody } else { &mut $demand.required_continuations })
        } else {
            ContinuationCustody::Caller(&mut $demand.required_continuations,
                &mut $demand.producer_contacts_in_this_demand)
        };
        match resume_held_upstream(
            $phase,
            $runtime,
            $principal,
            $request_scope,
            $wave.branch,
            custody,
            &head,
            $admission,
            $performed,
        ) {
            Ok(HeldUpstream::Ready(ready)) => {
                $runtime.output_demands.clear_required_stop(&head);
                if committed_ready(&ready, $admission)? {
                    $wave = reselect_required_wave($runtime, $wave, $admission)?;
                    $resolved_on_wave.clear();
                    $queue.wave_moved();
                }
                // Certify the same row again against the finished upstream.
                continue $label;
            }
            Ok(HeldUpstream::Unfinished) => {
                if $queue.active() {
                    $hold_queue_frame!($label, None)
                }
                $finish_caller!($label, WorthQueryOutputDemandAdvance::Pending)
            }
            // The Ready the superseded successor replaced answers again.
            Ok(HeldUpstream::GaveBack) => continue $label,
            Err(stop) => $stopped!($label, &head, stop),
        }
    }};
}
