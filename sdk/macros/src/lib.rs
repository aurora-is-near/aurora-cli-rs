use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Fields, Meta, Type};

/// Derive macro for implementing the ContractMethod trait
///
/// Usage:
/// ```rust
/// #[derive(ContractMethod)]
/// #[contract_method(method = "some_method", response = (), serialize_as = "json")]
/// struct MyMethod {
///     #[contract_param]
///     pub field1: String, // Serialized
///     #[contract_param]
///     pub raw_data: Vec<u8>, // Returned as-is
///     pub field2: String, // Ignored
/// }
/// ```
#[proc_macro_derive(ContractMethod, attributes(contract_method, contract_param))]
pub fn derive_contract_method(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    // Parse the contract_method attribute
    let contract_method_attr = input
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("contract_method"))
        .expect("contract_method attribute is required");

    let mut method_name = None;
    let mut response_type = None;
    let mut serialize_as = "borsh".to_string(); // default to borsh

    // Parse the attribute arguments manually
    if let Meta::List(meta_list) = &contract_method_attr.meta {
        // Convert tokens to string and parse manually
        let tokens_str = meta_list.tokens.to_string();

        // Simple parsing - split by comma and process each part
        for part in tokens_str.split(',') {
            let part = part.trim();

            if let Some(eq_pos) = part.find('=') {
                let key = part[..eq_pos].trim();
                let value = part[eq_pos + 1..].trim();

                match key {
                    "method" => {
                        // Remove quotes
                        let value = value.trim_matches('"');
                        method_name = Some(value.to_string());
                    }
                    "response" => {
                        // Parse the response type from string
                        if let Ok(parsed_type) = syn::parse_str::<syn::Type>(value) {
                            response_type = Some(quote! { #parsed_type });
                        }
                    }
                    "serialize_as" => {
                        let value = value.trim_matches('"');
                        serialize_as = value.to_string();
                    }
                    _ => {}
                }
            }
        }
    }

    let method_name = method_name.expect("method attribute is required");
    let response_type = response_type.expect("response attribute is required");

    // Find the field marked with contract_param (only one allowed)
    let param_field = match &input.data {
        syn::Data::Struct(data_struct) => {
            match &data_struct.fields {
                Fields::Named(fields_named) => {
                    let mut param_field = None;

                    for field in &fields_named.named {
                        // Check if field has contract_param attribute
                        let has_contract_param = field
                            .attrs
                            .iter()
                            .any(|attr| attr.path().is_ident("contract_param"));

                        if has_contract_param {
                            if param_field.is_some() {
                                panic!("Only one field can be marked with #[contract_param]");
                            }
                            if let Some(field_name) = &field.ident {
                                param_field = Some((field_name, &field.ty));
                            }
                        }
                    }

                    param_field
                }
                Fields::Unit => None, // Unit struct - no fields
                Fields::Unnamed(_) => {
                    // For tuple structs, return None (empty params)
                    None
                }
            }
        }
        syn::Data::Union(_) => None,
        _ => {
            panic!("ContractMethod can only be derived for structs or unions");
        }
    };

    // Helper function to check if type is Vec<u8>
    fn is_vec_u8(ty: &Type) -> bool {
        if let Type::Path(type_path) = ty {
            if let Some(segment) = type_path.path.segments.last() {
                if segment.ident == "Vec" {
                    if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                        if let Some(syn::GenericArgument::Type(Type::Path(inner_path))) =
                            args.args.first()
                        {
                            if let Some(inner_segment) = inner_path.path.segments.last() {
                                return inner_segment.ident == "u8";
                            }
                        }
                    }
                }
            }
        }
        false
    }

    // Generate the params() method
    let params_impl = if let Some((field_name, field_type)) = param_field {
        if is_vec_u8(field_type) {
            // Vec<u8> field - return as is
            quote! {
                fn params(&self) -> Result<Vec<u8>, std::io::Error> {
                    Ok(self.#field_name.clone())
                }
            }
        } else {
            // Other types - serialize based on serialize_as
            match serialize_as.as_str() {
                "json" => quote! {
                    fn params(&self) -> Result<Vec<u8>, std::io::Error> {
                        serde_json::to_vec(&self.#field_name).map_err(Into::into)
                    }
                },
                "borsh" => quote! {
                    fn params(&self) -> Result<Vec<u8>, std::io::Error> {
                        borsh::to_vec(&self.#field_name).map_err(Into::into)
                    }
                },
                _ => panic!("serialize_as must be either 'json' or 'borsh'"),
            }
        }
    } else {
        // No field marked or unit struct - return empty vector
        quote! {
            fn params(&self) -> Result<Vec<u8>, std::io::Error> {
                Ok(Vec::new())
            }
        }
    };

    // Try to determine if we're inside the crate or outside
    let crate_name = std::env::var("CARGO_PKG_NAME").unwrap_or_default();
    let target_name = std::env::var("CARGO_BIN_NAME")
        .or_else(|_| std::env::var("CARGO_EXAMPLE_NAME"))
        .unwrap_or_default();

    let trait_path = if crate_name == "aurora-sdk-rs" && target_name.is_empty() {
        // We're inside the main crate, not in an example
        quote! { crate::aurora::ContractMethod }
    } else {
        // We're in an example or external crate
        quote! { aurora_sdk_rs::aurora::ContractMethod }
    };

    let expanded = quote! {
        impl #impl_generics #trait_path for #name #ty_generics #where_clause {
            type Response = #response_type;

            fn method_name(&self) -> &'static str {
                #method_name
            }

            #params_impl
        }
    };

    TokenStream::from(expanded)
}
