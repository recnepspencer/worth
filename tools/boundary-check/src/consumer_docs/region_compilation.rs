//! Configuration on actual Rust items containing or contained by a marked region.
use super::native_test_cfg;
use syn::{
    spanned::Spanned,
    visit::{self, Visit},
    Attribute, Item,
};

pub(super) fn validate(
    source: &syn::File,
    first_line: usize,
    last_line: usize,
) -> Result<(), String> {
    let mut region = Region {
        first_line,
        last_line,
        refusal: None,
    };
    region.visit_file(source);
    region.refusal.map_or(Ok(()), Err)
}

struct Region {
    first_line: usize,
    last_line: usize,
    refusal: Option<String>,
}

impl Region {
    fn intersects(&self, value: &impl Spanned) -> bool {
        value.span().start().line <= self.last_line && value.span().end().line >= self.first_line
    }
    fn require_active(&mut self, attributes: &[Attribute]) {
        match native_test_cfg::active(attributes) {
            Ok(true) => {}
            Ok(false) => {
                self.refusal =
                    Some("compiled region contains or belongs to an inactive Rust item".into())
            }
            Err(error) => self.refusal = Some(error),
        }
    }
}

impl<'ast> Visit<'ast> for Region {
    fn visit_item(&mut self, item: &'ast Item) {
        if self.intersects(item) {
            let attributes = match item {
                Item::Const(item) => &item.attrs,
                Item::Enum(item) => &item.attrs,
                Item::ExternCrate(item) => &item.attrs,
                Item::Fn(item) => &item.attrs,
                Item::ForeignMod(item) => &item.attrs,
                Item::Impl(item) => &item.attrs,
                Item::Macro(item) => &item.attrs,
                Item::Mod(item) => &item.attrs,
                Item::Static(item) => &item.attrs,
                Item::Struct(item) => &item.attrs,
                Item::Trait(item) => &item.attrs,
                Item::TraitAlias(item) => &item.attrs,
                Item::Type(item) => &item.attrs,
                Item::Union(item) => &item.attrs,
                Item::Use(item) => &item.attrs,
                _ => {
                    self.refusal = Some("unsupported Rust item in compiled region".into());
                    return;
                }
            };
            self.require_active(attributes);
        }
        visit::visit_item(self, item);
    }

    fn visit_attribute(&mut self, attribute: &'ast Attribute) {
        if self.intersects(attribute) {
            self.require_active(std::slice::from_ref(attribute));
        }
        visit::visit_attribute(self, attribute);
    }

    fn visit_impl_item_fn(&mut self, item: &'ast syn::ImplItemFn) {
        if self.intersects(item) {
            self.require_active(&item.attrs);
        }
        visit::visit_impl_item_fn(self, item);
    }

    fn visit_trait_item_fn(&mut self, item: &'ast syn::TraitItemFn) {
        if self.intersects(item) {
            self.require_active(&item.attrs);
        }
        visit::visit_trait_item_fn(self, item);
    }

    fn visit_impl_item_const(&mut self, item: &'ast syn::ImplItemConst) {
        if self.intersects(item) {
            self.require_active(&item.attrs);
        }
        visit::visit_impl_item_const(self, item);
    }

    fn visit_trait_item_const(&mut self, item: &'ast syn::TraitItemConst) {
        if self.intersects(item) {
            self.require_active(&item.attrs);
        }
        visit::visit_trait_item_const(self, item);
    }

    fn visit_local(&mut self, local: &'ast syn::Local) {
        if self.intersects(local) {
            self.require_active(&local.attrs);
        }
        visit::visit_local(self, local);
    }

    fn visit_expr_block(&mut self, block: &'ast syn::ExprBlock) {
        if self.intersects(block) {
            self.require_active(&block.attrs);
        }
        visit::visit_expr_block(self, block);
    }
}
