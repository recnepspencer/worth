use store_m4_dx_spike::*;

fn head_view_cannot_scan_history(head: &HeadOnlyView<'_, MySchema>, req: &Request<'_, MySchema>) {
    let _ = head.scan_history::<AuditEntry>(req, AuditQuery::ordered());
}

fn main() {}
