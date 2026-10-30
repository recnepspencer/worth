use worth_server::{
    WorthServerProductOperationBasisKind, WorthServerProductOperationDeclaration,
    WorthServerProductOperationSupportSnapshot,
};

fn main() {
    let _digest_only_declaration = WorthServerProductOperationDeclaration::product_read(
        "product.connection.inspect",
        "product.connection.inspect.v1",
        WorthServerProductOperationBasisKind::DurableProductDerived,
        WorthServerProductOperationSupportSnapshot::production_admitted("inspect-supported"),
    );
}
