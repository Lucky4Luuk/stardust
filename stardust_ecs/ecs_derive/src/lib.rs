extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;

#[proc_macro_derive(EngineComponentName)]
pub fn engine_component_name_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();

    impl_engine_component_name(&ast)
}

fn impl_engine_component_name(ast: &syn::DeriveInput) -> TokenStream {
    let comp_name = &ast.ident;
    let gen = quote! {
        impl crate::EngineComponentName for #comp_name {
            fn name() -> &'static str { stringify!(comp_name) }
        }
    };
    gen.into()
}

#[proc_macro_derive(EngineComponent, attributes(editable, visible))]
pub fn engine_component_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();

    impl_engine_component(&ast)
}

enum VisAttribute {
    Hidden,
    Visible(Option<String>),
    Editable(Option<String>),
}

fn handle_path(path: syn::Path, modifier: &mut VisAttribute) {
    if let Some(ident) = path.get_ident() {
        if ident == "editable" {
            *modifier = VisAttribute::Editable(None);
        } else if ident == "visible" {
            *modifier = VisAttribute::Visible(None);
        }
    }
}

fn impl_engine_component(ast: &syn::DeriveInput) -> TokenStream {
    let comp_name = &ast.ident;
    let comp_name_styled = syn::Ident::new(&format!("{}", comp_name).replace("Comp", ""), proc_macro2::Span::call_site());
    if let syn::Data::Struct(struct_data) = &ast.data {
        if let syn::Fields::Named(fields) = &struct_data.fields {
            let mut editable_field_names = Vec::new();
            let mut editable_field_display_names = Vec::new();
            let mut visible_field_names = Vec::new();
            let mut visible_field_display_names = Vec::new();
            for field in &fields.named {
                let mut modifier = VisAttribute::Hidden;
                for attr in &field.attrs {
                    match attr.parse_meta() {
                        Ok(syn::Meta::Path(path)) => handle_path(path, &mut modifier),
                        Ok(syn::Meta::List(list)) => {
                            handle_path(list.path, &mut modifier);
                            match &mut modifier {
                                VisAttribute::Visible(name) | VisAttribute::Editable(name) => {
                                    if let Some(syn::NestedMeta::Lit(new_name_literal)) = list.nested.iter().next() {
                                        match new_name_literal {
                                            syn::Lit::Str(new_name) => *name = Some(new_name.value()),
                                            _ => panic!("Unsupported type!"),
                                        }
                                    } else {
                                        panic!("Unsupported name!");
                                    }
                                },
                                _ => {},
                            }
                        },
                        _ => unimplemented!(),
                    }
                }
                match modifier {
                    VisAttribute::Editable(name) => {
                        editable_field_names.push(field.ident.clone().unwrap());
                        editable_field_display_names.push(name.map(|name| {
                            syn::Ident::new(&name, proc_macro2::Span::call_site())
                        }).unwrap_or(field.ident.clone().unwrap()));
                    },
                    VisAttribute::Visible(name) => {
                        visible_field_names.push(field.ident.clone().unwrap());
                        visible_field_display_names.push(name.map(|name| {
                            syn::Ident::new(&name, proc_macro2::Span::call_site())
                        }).unwrap_or(field.ident.clone().unwrap()));
                    },
                    _ => {},
                }
            }
            let editable_field_name = editable_field_names.iter();
            let editable_field_display_name = editable_field_display_names.iter();
            let visible_field_name = visible_field_names.iter();
            let visible_field_display_name = visible_field_display_names.iter();
            let gen = quote! {
                impl crate::EngineComponentName for #comp_name {
                    fn name() -> &'static str { stringify!(#comp_name_styled) }
                }
                impl crate::EngineComponentGetField for #comp_name {
                    fn fields(&mut self) -> FieldMap {
                        use crate::fields::ToValue;
                        let mut map = FieldMap::new();
                        #(
                            map.insert(stringify!(#editable_field_display_name).to_string(), (true, self.#editable_field_name.to_value()));
                        )*
                        #(
                            map.insert(stringify!(#visible_field_display_name).to_string(), (false, self.#visible_field_name.to_value()));
                        )*
                        map
                    }
                }
                impl crate::EngineComponentImplementor for #comp_name {}
            };
            gen.into()
        } else {
            panic!("Unsupported");
        }
    } else {
        panic!("Unsupported!");
    }
}
