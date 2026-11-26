//! The posting access the stock actions share, declared once here and
//! included by path from their sibling modules. Member paths are
//! `crate::`-qualified because they resolve where a fragment is included.
use worth_query_decl::facade::worth_query_access_fragment;

worth_query_access_fragment! {
    /// A posting resolves its bin by key and moves its on-hand quantity.
    stock_posting_access {
        read_entities: [crate::access_fragment_dx::schema::Bin];
        read_fields: [
            crate::access_fragment_dx::schema::BinFacts::bin_id(),
            crate::access_fragment_dx::schema::BinFacts::on_hand(),
        ];
        writes: [crate::access_fragment_dx::schema::BinFacts::on_hand()];
    }
}
pub(crate) use stock_posting_access;

worth_query_access_fragment! {
    /// Records a movement linked from its bin. It repeats the on-hand read
    /// and write of `stock_posting_access`, which register once.
    movement_record_access {
        read_fields: [crate::access_fragment_dx::schema::BinFacts::on_hand()];
        writes: [
            crate::access_fragment_dx::schema::BinFacts::on_hand(),
            crate::access_fragment_dx::schema::MovementFacts::movement_id(),
            crate::access_fragment_dx::schema::MovementFacts::moved(),
        ];
        creates: [crate::access_fragment_dx::schema::Movement];
        links: [crate::access_fragment_dx::schema::BinMovements];
    }
}
pub(crate) use movement_record_access;
