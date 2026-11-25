use store_m4_dx_spike::*;

fn borrowed_row_cannot_outlive_cursor<'a>(
    product: &ProductRef<'a, MySchema>,
    req: &Request<'_, MySchema>,
) -> Option<BorrowedHistoricalRow<'a, AuditEntry>> {
    let head = product.observe(req).ok()?;
    let cursor = product
        .at_revision(req, head.revision)
        .ok()?
        .scan_history::<AuditEntry>(req, AuditQuery::ordered())
        .ok()?;
    let borrowed = cursor.peek_borrowed_row();
    drop(cursor);
    borrowed
}

fn main() {}
