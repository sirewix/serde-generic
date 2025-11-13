extern crate proc_macro;
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::format_ident;
use quote::quote;
use serde::Ctxt;
use syn::Data;
use syn::DeriveInput;
use syn::Fields;
use syn::parse_macro_input;
use syn::punctuated::Punctuated;
use syn::token::Comma;

mod serde;

fn mk_idx(i: usize) -> TokenStream2 {
    (0..i).fold(
        quote! {::serde_generic::Zero},
        |tail, _| quote! {::serde_generic::Succ<#tail>},
    )
}

fn mk_type_var(i: usize) -> TokenStream2 {
    let idx = mk_idx(i);
    quote! {::serde_generic::TypeVar<#idx>}
}

#[proc_macro_derive(SerdeGeneric, attributes(serde))]
pub fn derive_serde_generic(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let type_ident = &input.ident;

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let type_params = input
        .generics
        .params
        .iter()
        .rfold(quote! {()}, |tail, field| quote! {(#field, #tail)});
    let mock_params = input
        .generics
        .type_params()
        .enumerate()
        .map(|(i, _)| mk_type_var(i));
    let cx = serde::Ctxt::new();
    let container_attrs = serde::attr::Container::from_ast(&cx, &input);

    assert!(
        container_attrs.type_from().is_none()
            && container_attrs.type_try_from().is_none()
            && container_attrs.type_into().is_none()
            && container_attrs.remote().is_none(),
        "serde attributes `type_from`, `type_try_from`, `type_into` and `remote` are not supported",
    );

    let container_default = container_attrs.default();
    let rename_rules = container_attrs.rename_all_rules();
    let container_def = container_attrs.to_serde_generic_term_repr();

    let ((repr, other_impls), to_repr, from_repr) = match &input.data {
        Data::Struct(str) => match &str.fields {
            Fields::Named(fields) => {
                let struct_common = for_a_struct(
                    &cx,
                    &container_default,
                    rename_rules,
                    &type_ident,
                    &ty_generics,
                    &impl_generics,
                    quote! {::serde_generic::NamedStruct},
                    &fields.named,
                );
                let idents = fields
                    .named
                    .iter()
                    .map(|field| field.ident.as_ref().unwrap());

                let to_repr = idents.clone().rfold(
                    quote! {::serde_generic::HNil},
                    |tail, field_name| quote! {::serde_generic::HCons(self.#field_name, #tail)},
                );
                let from_repr_pat = idents.clone().rfold(
                    quote! {::serde_generic::HNil},
                    |tail, field_name| quote! {::serde_generic::HCons(#field_name, #tail)},
                );
                let from_repr = quote! {let ::serde_generic::NamedStruct(#from_repr_pat) = repr; Self { #(#idents,)* } };
                (
                    struct_common,
                    quote! {::serde_generic::NamedStruct(#to_repr)},
                    from_repr,
                )
            }
            Fields::Unnamed(fields) => {
                let struct_common = for_a_struct(
                    &cx,
                    &container_default,
                    rename_rules,
                    &type_ident,
                    &ty_generics,
                    &impl_generics,
                    quote! {::serde_generic::UnnamedStruct},
                    &fields.unnamed,
                );
                let indexes = fields.unnamed.iter().enumerate();

                let to_repr = indexes.clone().map(|(i, _)| syn::Index::from(i)).rfold(
                    quote! {::serde_generic::HNil},
                    |tail, i| quote! {::serde_generic::HCons(self.#i, #tail)},
                );

                let idents = indexes.map(|(i, _)| format_ident!("x{i}"));

                let from_repr_pat = idents.clone().rfold(
                    quote! {::serde_generic::HNil},
                    |tail, i| quote! {::serde_generic::HCons(#i, #tail)},
                );

                let from_repr = quote! {let ::serde_generic::UnnamedStruct(#from_repr_pat) = repr; Self ( #(#idents,)* ) };
                (
                    struct_common,
                    quote! {::serde_generic::UnnamedStruct(#to_repr)},
                    from_repr,
                )
            }
            Fields::Unit => (
                (
                    quote! {::serde_generic::UnitStruct},
                    Box::new(std::iter::empty()) as Box<dyn Iterator<Item = TokenStream2>>,
                ),
                quote! {::serde_generic::UnitStruct},
                quote! {Self},
            ),
        },
        Data::Enum(variants) => for_an_enum(
            &cx,
            &container_default,
            rename_rules,
            &type_ident,
            &ty_generics,
            &impl_generics,
            variants,
        ),
        Data::Union(_) => panic!("Union types are not supported"),
    };

    let res = quote! {
      #[automatically_derived]
      impl #impl_generics  ::serde_generic::SerdeGeneric for #type_ident #ty_generics #where_clause {
        type Params = #type_params;
        type Mocked = #type_ident <#(#mock_params,)*>;
        type Repr = #repr;
        fn to_repr(self) -> Self::Repr { #to_repr }
        fn from_repr(repr: Self::Repr) -> Self { #from_repr }
        const CONTAINER: ::serde_generic::serde::Container<Self> = #container_def;
      }
      #(#other_impls)*
    }
    .into();

    cx.check().unwrap();
    res
}

fn for_a_struct<'a>(
    cx: &'a Ctxt,
    container_default: &'a serde::attr::Default,
    rename_rules: serde::attr::RenameAllRules,
    type_ident: &'a syn::Ident,
    ty_generics: &'a syn::TypeGenerics<'a>,
    impl_generics: &'a syn::ImplGenerics<'a>,
    wrapper: TokenStream2,
    fields: &'a Punctuated<syn::Field, Comma>,
) -> (TokenStream2, Box<dyn Iterator<Item = TokenStream2> + 'a>) {
    let types_hlist = fields.iter().map(|field| &field.ty).rfold(
        quote! {::serde_generic::HNil},
        |tail, field| quote! {::serde_generic::HCons<#field, #tail>},
    );

    let number_of_fields = fields.len();
    let other_impls = fields
        .iter()
        .scan(number_of_fields + 1, |i, x| {
            *i -= 1;
            Some((*i, x))
        })
        .enumerate()
        .map(move |(i, (i_rev, field))| {
            let idx = mk_idx(i_rev);
            let field_type = &field.ty;
            let serde_field_attr =
                serde::attr::Field::from_ast(&cx, i, &field, None, &container_default)
                    .mutate(|f| f.rename_by_rules(rename_rules))
                    .to_serde_generic_term_repr();
            quote! {
                #[automatically_derived]
                impl #impl_generics ::serde_generic::SerdeFieldAttr<#field_type, #idx>
                   for #type_ident #ty_generics {
                   const FIELD: ::serde_generic::serde::Field<Self, #field_type> = #serde_field_attr;
                }
            }
        });
    let repr = quote! {#wrapper <#types_hlist>};
    (repr, Box::new(other_impls))
}

trait GenericCombinator: Sized {
    fn mutate(mut self, f: impl FnOnce(&mut Self)) -> Self {
        f(&mut self);
        self
    }
}
impl<X: Sized> GenericCombinator for X {}

fn for_an_enum<'a>(
    cx: &'a Ctxt,
    container_default: &'a serde::attr::Default,
    rename_rules: serde::attr::RenameAllRules,
    type_ident: &'a syn::Ident,
    ty_generics: &'a syn::TypeGenerics<'a>,
    impl_generics: &'a syn::ImplGenerics<'a>,
    data_enum: &'a syn::DataEnum,
) -> (
    (TokenStream2, Box<dyn Iterator<Item = TokenStream2> + 'a>),
    TokenStream2,
    TokenStream2,
) {
    let repr = data_enum
        .variants
        .iter()
        .map(|variant| &variant.fields)
        .rfold(quote! {::serde_generic::HSumNil}, |tail, var_fields| {
            let var_struct = var_fields.iter().map(|field| &field.ty).rfold(
                quote! {::serde_generic::HNil},
                |tail, field| quote! {::serde_generic::HCons<#field, #tail>},
            );

            match &var_fields {
                Fields::Named(_) => {
                    quote! {::serde_generic::HSum<::serde_generic::NamedStruct<#var_struct>, #tail>}
                }
                Fields::Unnamed(_) => {
                    quote! {::serde_generic::HSum<::serde_generic::UnnamedStruct<#var_struct>, #tail>}
                }
                Fields::Unit => quote! {::serde_generic::HSum<::serde_generic::UnitStruct, #tail>},
            }
        });

    let number_of_variants = data_enum.variants.len();
    let other_impls = data_enum
        .variants
        .iter()
        .scan(number_of_variants + 1, |i, x| {
            *i -= 1;
            Some((*i, x))
        })
        .map(move |(i_rev, variant)| {
            let idx = mk_idx(i_rev);
            let serde_var_attr = serde::attr::Variant::from_ast(&cx, &variant)
                    .mutate(|f| f.rename_by_rules(rename_rules));

            let number_of_fields = variant.fields.len();
            let field_impls = variant
                .fields
                .iter()
                .scan(number_of_fields + 1, |j, x| {
                    *j -= 1;
                    Some((*j, x))
                })
                .enumerate()
                .map(|(j, (j_rev, field))| {
                    let jdx = mk_idx(j_rev);
                    let field_type = &field.ty;
                    let serde_field_attr = serde::attr::Field::from_ast(
                        &cx,
                        j,
                        &field,
                        Some(&serde_var_attr),
                        &container_default,
                    )
                    .mutate(|f| f.rename_by_rules(rename_rules))
                    .to_serde_generic_term_repr();
                    quote! {
                        #[automatically_derived]
                        impl #impl_generics ::serde_generic::SerdeVariantFieldAttr<#field_type, #idx, #jdx>
                            for #type_ident #ty_generics {
                            const FIELD: ::serde_generic::serde::Field<Self, #field_type> = #serde_field_attr;
                        }
                    }
                });

            let serde_var_attr = serde_var_attr.to_serde_generic_term_repr();
            quote! {
                #[automatically_derived]
                impl #impl_generics ::serde_generic::SerdeVariantAttr<#idx> for #type_ident #ty_generics {
                   const VARIANT: ::serde_generic::serde::Variant = #serde_var_attr;
                }
                #(#field_impls)*
            }
        });

    let to_from_repr = data_enum.variants.iter().enumerate().map(|(i, variant)| {
        let var_ident = &variant.ident;
        let (orig, generic) = match &variant.fields {
            Fields::Named(fields) => {
                let field_idents = fields
                    .named
                    .iter()
                    .map(|field| field.ident.as_ref().unwrap());
                let cons = field_idents.clone().rfold(
                    quote! {::serde_generic::HNil},
                    |tail, field_ident| quote! {::serde_generic::HCons(#field_ident, #tail)},
                );
                (
                    quote! {{#(#field_idents,)*}},
                    quote! {::serde_generic::NamedStruct(#cons)},
                )
            }
            Fields::Unnamed(fields) => {
                let indices = fields
                    .unnamed
                    .iter()
                    .enumerate()
                    .map(|(j, _)| format_ident!("x{j}"));
                let cons = indices.clone().rfold(
                    quote! {::serde_generic::HNil},
                    |tail, j| quote! {::serde_generic::HCons(#j, #tail)},
                );
                (
                    quote! {(#(#indices,)*)},
                    quote! {::serde_generic::UnnamedStruct(#cons)},
                )
            }
            Fields::Unit => (quote! {}, quote! {::serde_generic::UnitStruct}),
        };
        let generic = (0..i).fold(
            quote! {::serde_generic::HSum::L(#generic)},
            |tail, _| quote! {::serde_generic::HSum::R(#tail)},
        );
        (var_ident, orig, generic)
    });

    let to_repr_matches = to_from_repr
        .clone()
        .map(|(var_ident, orig, generic)| quote! {Self::#var_ident #orig => #generic});
    let to_repr = quote! { ::serde_generic::Enum(match self { #(#to_repr_matches,)* }) };

    let from_repr_matches =
        to_from_repr.map(|(var_ident, orig, generic)| quote! {#generic => Self::#var_ident #orig });
    let from_repr = quote! { match repr.0 { #(#from_repr_matches,)* } };

    (
        (quote! {::serde_generic::Enum<#repr>}, Box::new(other_impls)),
        to_repr,
        from_repr,
    )
}
