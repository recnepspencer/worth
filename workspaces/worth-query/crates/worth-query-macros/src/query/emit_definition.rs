//! Hygienic lowering of `#[derive(WorthQuery)]` definition builders,
//! comparison predicates, related-record predicates, root paths, ordering,
//! whole-result live mode, and managed-index query intent association.
use super::defaults::{basis_expr, cardinality_expr, disclosure_expr, lanes_expr};
use super::emit_predicates::{emit_filter_block, emit_related_predicate, partition_predicates};
use super::grammar::QueryDeclaration;
use super::hygiene::HygienicNames;
use super::predicates::{LiveMode, OrderDirection, PredicateRhs, TraversalDirection};
use crate::facade_paths::{declaration, fragment};
use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use syn::DeriveInput;

pub(super) fn managed_index_intent_impl(
    input: &DeriveInput,
    decl_spec: &QueryDeclaration,
    names: &HygienicNames,
) -> Option<TokenStream> {
    let mi = decl_spec.managed_index.as_ref()?;
    let (decl, request) = (declaration(), &input.ident);
    let (schema, idx) = (&decl_spec.schema, &mi.index);
    let (schema_alias, index_alias) = (&names.schema_alias, &names.index_alias);
    Some(quote! {
        const _: () = {
            type #schema_alias = #schema;
            type #index_alias = #idx;
            impl #decl::application_query::ApplicationManagedIndexQueryIntent<#schema_alias>
                for #request
            {
                type Index = #index_alias;
            }
        };
    })
}

pub(super) fn definition_builder(
    input: &DeriveInput,
    decl_spec: &QueryDeclaration,
    names: &HygienicNames,
) -> Option<TokenStream> {
    let def = decl_spec.definition.as_ref()?;
    let (decl, frag, vis) = (declaration(), fragment(), &input.vis);
    let (schema, query, result) = (&decl_spec.schema, &decl_spec.query, &decl_spec.result);
    let root = &def.root;
    let aq = quote!(#decl::application_query);
    let aschema = quote!(#decl::application_schema);
    let (schema_alias, result_alias, root_alias) =
        (&names.schema_alias, &names.result_alias, &names.root_alias);
    let (shape_local, builder_local, clauses_local, related_local, encode_lit_fn) = (
        &names.shape_local,
        &names.builder_local,
        &names.clauses_local,
        &names.related_local,
        &names.encode_lit_fn,
    );

    let param_val = quote!(
        <<Self as #aq::ApplicationQueryMarkerIdentity<#schema_alias>>::ParameterBinding
            as #aschema::ApplicationStructuredValueBinding>::Value
    );
    let result_val = quote!(<#result_alias as #aschema::ApplicationStructuredValueBinding>::Value);
    let scope_entity = quote!(<Self as #aq::ApplicationQueryMarkerIdentity<#schema_alias>>::Scope);

    let cardinality = cardinality_expr(def.cardinality.as_ref());
    let disclosure = disclosure_expr(def.disclosure.as_ref());
    let basis = basis_expr(def.basis_support.as_ref());
    let lanes = match (def.lanes.as_ref(), def.live) {
        (None, Some(LiveMode::WholeResult)) => {
            quote!(#aq::ApplicationQueryLaneEligibility::one_shot().with_live())
        }
        (explicit, _) => lanes_expr(explicit),
    };
    let live_step = (def.live == Some(LiveMode::WholeResult)).then(|| quote!(.live_whole_result()));
    let auth = def.ability.as_ref().map_or_else(
        || quote!(.public()),
        |ability| quote!(.requires_ability(#ability)),
    );
    let params = decl_spec
        .params
        .iter()
        .filter(|param| !param.is_probe)
        .map(|param| {
            let selector = &param.selector;
            quote!(.parameter(Self::#selector()))
        });
    let managed_index_step = decl_spec.managed_index.as_ref().and_then(|mi| {
        let probe = decl_spec.params.iter().find(|param| param.is_probe)?;
        let selector = &probe.selector;
        let idx = &mi.index;
        let method = if mi.refined {
            quote!(managed_index_refined)
        } else {
            quote!(managed_index)
        };
        Some(quote! {
            .#method(
                <#idx as #aq::ApplicationManagedIndexBinding<#schema_alias>>::reference(),
                Self::#selector(),
            )
        })
    });

    let root_path_steps: Vec<TokenStream> = def
        .root_paths
        .iter()
        .map(|path| {
            let hops = path.hops.iter().map(|hop| {
                let rel = &hop.relation;
                match hop.direction {
                    TraversalDirection::Forward => quote!(.forward(<#rel>::reference())),
                    TraversalDirection::Reverse => quote!(.reverse(<#rel>::reference())),
                }
            });
            quote_spanned! {path.span=>
                .root_path(
                    #aq::ApplicationQueryRootPath::from(
                        #aschema::ApplicationEntityRef::<#schema_alias, #scope_entity>::from_schema_identifier(
                            <#scope_entity as #frag::Entity>::NAME,
                        ),
                    )
                    #(#hops)*
                )
            }
        })
        .collect();

    let mut structured_eq_steps = Vec::new();
    let mut filter_predicates = Vec::new();
    partition_predicates(
        &def.predicates,
        encode_lit_fn,
        &mut structured_eq_steps,
        &mut filter_predicates,
    );
    let root_filter_block = emit_filter_block(
        &filter_predicates,
        builder_local,
        clauses_local,
        encode_lit_fn,
        &aq,
    );
    let related_blocks: Vec<TokenStream> = def
        .related_predicates
        .iter()
        .map(|rel| {
            emit_related_predicate(
                rel,
                builder_local,
                related_local,
                clauses_local,
                encode_lit_fn,
                &aq,
            )
        })
        .collect();

    let order_steps: Vec<TokenStream> = def
        .order_by
        .iter()
        .map(|ord| {
            let slot = &ord.slot;
            let dir = match ord.direction {
                OrderDirection::Ascending => {
                    quote!(#aq::ApplicationQueryOrderingDirection::Ascending)
                }
                OrderDirection::Descending => {
                    quote!(#aq::ApplicationQueryOrderingDirection::Descending)
                }
            };
            quote_spanned!(ord.span=> .order_by(#slot, #dir))
        })
        .collect();

    let default_definition = (!def.explicit_shape).then(|| {
        quote! {
            /// Builds the query definition using the projection row's derived
            /// `result_shape()` and the query's declared defaults and predicates.
            #[allow(dead_code)]
            #vis fn definition() -> #aq::ApplicationQueryDefinitionBuilder<
                #schema_alias,
                Self,
                #param_val,
                #result_val,
                #scope_entity,
            > {
                Self::definition_with(<#result_val>::result_shape::<Self>())
            }
        }
    });

    let has_literal = def
        .predicates
        .iter()
        .chain(
            def.related_predicates
                .iter()
                .flat_map(|r| &r.target_predicates),
        )
        .any(|p| matches!(p.rhs, PredicateRhs::Literal(_)));
    let encode_lit_def = has_literal.then(|| {
        quote! {
            #[allow(dead_code)]
            fn #encode_lit_fn<__F>(
                _field: &__F,
                value: <<__F as #frag::RecordSelection>::Binding as #aschema::ApplicationScalarValueBinding>::Value,
            ) -> #aschema::ApplicationEncodedScalarValue<<__F as #frag::RecordSelection>::Binding>
            where
                __F: #frag::RecordSelection,
                __F::Binding: #aschema::ApplicationReadableScalarValueBinding,
            {
                #aschema::ApplicationEncodedScalarValue::<<__F as #frag::RecordSelection>::Binding>::try_new(value)
                    .expect("derived WorthQuery literal constant encodes")
            }
        }
    });

    Some(quote! {
        const _: () = {
            type #schema_alias = #schema;
            type #result_alias = #result;
            type #root_alias = #root;
            #encode_lit_def

            const _: () = {
                #[allow(dead_code, non_snake_case)]
                fn #shape_local() {}
                #[allow(dead_code, non_snake_case)]
                fn #builder_local() {}
                #[allow(dead_code, non_snake_case)]
                fn #clauses_local() {}
                #[allow(dead_code, non_snake_case)]
                fn #related_local() {}

                impl #query {
                    #default_definition

                    /// Builds the query definition with an explicit result shape and
                    /// the query's declared defaults, parameters, and predicates.
                    #[allow(dead_code)]
                    #vis fn definition_with(
                        #shape_local: #aq::TypedApplicationQueryResultShape<
                            #schema_alias,
                            Self,
                            #root_alias,
                            #result_val,
                            #result_alias,
                        >,
                    ) -> #aq::ApplicationQueryDefinitionBuilder<
                        #schema_alias,
                        Self,
                        #param_val,
                        #result_val,
                        #scope_entity,
                    > {
                        #[allow(unused_mut)]
                        let mut #builder_local = #aq::ApplicationQueryDefinitionBuilder::declare(Self::reference())
                            .root(#aschema::ApplicationEntityRef::from_schema_identifier(
                                <#root_alias as #frag::Entity>::NAME,
                            ))
                            .scope(#aschema::ApplicationEntityRef::from_schema_identifier(
                                <#scope_entity as #frag::Entity>::NAME,
                            ))
                            .result_shape(#shape_local)
                            .cardinality(#cardinality)
                            .disclosure(#disclosure)
                            .basis_support(#basis)
                            .lanes(#lanes)
                            #auth
                            #live_step
                            #(#params)*
                            #managed_index_step
                            #(#root_path_steps)*
                            #(#structured_eq_steps)*;
                        #root_filter_block
                        #(#related_blocks)*
                        #builder_local #(#order_steps)*
                    }
                }
            };
        };
    })
}
