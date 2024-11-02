use proc_macro::TokenStream;
use proc_macro2::{Literal, TokenTree};
use proc_macro_error::{abort, proc_macro_error};
use quote::quote;
use syn::{parse_macro_input, spanned::Spanned, Data, DeriveInput, Type};

/// Macro to mark something affected by opacity.
///
/// # Field Attributes
///
/// * `#[opacity]`
///
///   Makes `bevy_mod_opacity` set its value as alpha,
///   valid on `f32` or bevy's color types.
///
/// # Type Attributes
///
/// * `#[opacity(asset)]`
///
///   Registers `Handle<Self>`.
///   
/// *  `#[opacity(extends = StandardMaterial)]`
///
///   Registers `Handle<ExtendedMaterial<Base, Self>>` where `Base` is also `OpacityAsset`.
///
/// *  `#[opacity(masks = StandardMaterial)]`
///
///   Registers `Handle<ExtendedMaterial<Base, Self>>` where the `Base` is not affected by opacity.
#[proc_macro_error]
#[proc_macro_derive(Opacity, attributes(opacity))]
pub fn opacity(tokens: TokenStream) -> TokenStream {
    let input = parse_macro_input!(tokens as DeriveInput);

    let mut asset = false;
    let mut extends = Vec::new();
    let mut masks = Vec::new();
    let mut fields = Vec::new();
    let name = input.ident;

    let Data::Struct(s) = input.data else {
        abort!(name.span(), "Only supports struct.")
    };
    match s.fields {
        syn::Fields::Named(fields_named) => {
            for field in fields_named.named {
                for attribute in field.attrs {
                    if attribute.path().is_ident("opacity") {
                        fields.push(TokenTree::Ident(field.ident.clone().unwrap()));
                    }
                }
            }
        }
        syn::Fields::Unnamed(fields_unnamed) => {
            for (index, field) in fields_unnamed.unnamed.into_iter().enumerate() {
                for attribute in field.attrs {
                    if attribute.path().is_ident("opacity") {
                        fields.push(TokenTree::Literal(Literal::usize_unsuffixed(index)));
                    }
                }
            }
        }
        syn::Fields::Unit => (),
    }

    for attribute in &input.attrs {
        if !attribute.path().is_ident("opacity") {
            continue;
        }
        #[allow(clippy::blocks_in_conditions)]
        if attribute
            .parse_nested_meta(|meta| {
                if meta.path.is_ident("asset") {
                    asset = true;
                } else if meta.path.is_ident("extends") {
                    extends.push(meta.value()?.parse::<Type>()?);
                } else if meta.path.is_ident("masks") {
                    masks.push(meta.value()?.parse::<Type>()?);
                } else {
                    abort!(meta.path.span(), "Expected 'asset', 'extends' or 'masks'.");
                }
                Ok(())
            })
            .is_err()
        {
            abort!(attribute.meta.span(), "Expected a type.")
        }
    }
    let crate0 = quote! {::bevy_mod_opacity};
    let mut result = quote! {
        const _: () =  {
            impl #crate0::OpacityComponent for #name {
                type Cx = ();

                fn apply_opacity(&mut self, _: &mut (), opacity: f32) {
                    #(#crate0::set_alpha(&mut self.#fields, opacity);)*
                }
            }
        };
    };
    if asset {
        result.extend(quote! {
            const _: () =  {
                impl #crate0::OpacityAsset for #name {
                    fn apply_opacity(&mut self, opacity: f32) {
                        #(#crate0::set_alpha(&mut self.#fields, opacity);)*
                    }
                }
            };
        });
    }
    for ty in extends {
        result.extend(quote! {
            const _: () =  {
                impl #crate0::OpacityMaterialExtension<#ty> for #name {
                    fn apply_opacity(a: &mut #ty, b: &mut Self, opacity: f32) {
                        #crate0::OpacityAsset::apply_opacity(a, opacity);
                        #(#crate0::set_alpha(&mut b.#fields, opacity);)*
                    }
                }
            };
        });
    }
    for ty in masks {
        result.extend(quote! {
            const _: () =  {
                impl #crate0::OpacityMaterialExtension<#ty> for #name {
                    fn apply_opacity(a: &mut #ty, b: &mut Self, opacity: f32) {
                        #(#crate0::set_alpha(&mut b.#fields, opacity);)*
                    }
                }
            };
        });
    }
    result.into()
}
