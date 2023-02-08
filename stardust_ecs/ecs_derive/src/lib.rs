extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;

#[proc_macro_derive(EngineComponent, attributes(editable, visible))]
pub fn engine_component_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();

    impl_engine_component(&ast)
}

fn impl_engine_component(ast: &syn::DeriveInput) -> TokenStream {
    let comp_name = &ast.ident;
    if let syn::Data::Struct(struct_data) = &ast.data {
        if let syn::Fields::Named(fields) = &struct_data.fields {
            let mut editable_field_names = Vec::new();
            let mut visible_field_names = Vec::new();
            for field in &fields.named {
                for attr in &field.attrs {
                    if let Ok(syn::Meta::Path(path)) = attr.parse_meta() {
                        if let Some(ident) = path.get_ident() {
                            if ident == "editable" {
                                editable_field_names.push(&field.ident);
                            } else if ident == "visible" {
                                visible_field_names.push(&field.ident);
                            }
                        }
                    }
                }
            }
            let editable_field_name = editable_field_names.iter();
            let gen = quote! {
                impl crate::EngineComponentName for #comp_name {
                    fn name() -> &'static str { stringify!(comp_name) }
                }
                impl crate::EngineComponent for #comp_name {
                    fn fields(&mut self) -> FieldMap {
                        let mut map = FieldMap::new();
                        #(
                            map.insert(stringify!(#editable_field_name).to_string(), (true, Value::from(&mut self.#editable_field_name)));
                        )*
                        map
                    }
                }
            };
            gen.into()
        } else {
            panic!("Unsupported");
        }
    } else {
        panic!("Unsupported!");
    }
}
