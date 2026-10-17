use syn::{Attribute, Data, DeriveInput, Ident, LitStr, Path};

pub(crate) struct ResidentValue {
    pub(crate) ident: Ident,
    pub(crate) data: Data,
    pub(crate) contract: Path,
}

impl ResidentValue {
    pub(crate) fn parse(input: DeriveInput) -> syn::Result<Self> {
        if !input.generics.params.is_empty() || input.generics.where_clause.is_some() {
            return Err(syn::Error::new_spanned(
                input.generics,
                "ChargedBytes derives concrete resident values; generic owners must declare their own ChargedBytes contract",
            ));
        }
        let mut contract = None;
        for attribute in input
            .attrs
            .iter()
            .filter(|attr| attr.path().is_ident("charged_bytes"))
        {
            attribute.parse_nested_meta(|meta| {
                if !meta.path.is_ident("crate") {
                    return Err(meta.error("unsupported ChargedBytes option; every resident field must count; only crate = \"path\" is supported"));
                }
                if contract.is_some() {
                    return Err(meta.error("duplicate ChargedBytes crate path"));
                }
                let literal: LitStr = meta.value()?.parse()?;
                contract = Some(literal.parse::<Path>()?);
                Ok(())
            })?;
        }
        match &input.data {
            Data::Struct(data) => {
                for field in &data.fields {
                    reject_field_options(&field.attrs)?;
                }
            }
            Data::Enum(data) => {
                for variant in &data.variants {
                    reject_field_options(&variant.attrs)?;
                    for field in &variant.fields {
                        reject_field_options(&field.attrs)?;
                    }
                }
            }
            Data::Union(data) => return Err(syn::Error::new_spanned(
                data.union_token,
                "ChargedBytes cannot identify a union's resident owner; declare an explicit owner ChargedBytes contract",
            )),
        }
        Ok(Self {
            ident: input.ident,
            data: input.data,
            contract: contract.unwrap_or_else(|| syn::parse_quote!(::worth_execution)),
        })
    }
}

fn reject_field_options(attributes: &[Attribute]) -> syn::Result<()> {
    if let Some(attribute) = attributes
        .iter()
        .find(|attr| attr.path().is_ident("charged_bytes"))
    {
        return Err(syn::Error::new_spanned(
            attribute,
            "ChargedBytes cannot skip or override resident fields; implement ChargedBytes on the named opaque/shared owner",
        ));
    }
    Ok(())
}
