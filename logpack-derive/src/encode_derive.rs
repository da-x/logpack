use proc_macro::TokenStream;
use syn;
use quote;

// TODO: use the likewise of serde-derive's with_bound so that 'T: Encoder' constraints
// will not be needed at struct definition.

pub fn derive(input: &TokenStream) -> quote::Tokens {
    let input: String = input.clone().to_string();
    let ast = syn::parse_macro_input(&input).expect("Couldn't parse item");
    let name = &ast.ident;
    let (_, ty_generics, where_clause) = ast.generics.split_for_impl();

    let encoder_fields = match ast.body {
        syn::Body::Enum(ref variants) => encoder_for_enum(name, &variants, false),
        syn::Body::Struct(ref variant_data) => encoder_for_struct(&variant_data, false),
    };

    let sizer_fields = match ast.body {
        syn::Body::Enum(ref variants) => encoder_for_enum(name, &variants, true),
        syn::Body::Struct(ref variant_data) => encoder_for_struct(&variant_data, true),
    };

    let result = quote! {
        impl #ty_generics ::logpack::Encoder for #name #ty_generics #where_clause {
            fn logpack_encode(&self, _buf: &mut ::logpack::buffers::BufEncoder) -> Result<(), (usize, usize)> {
                #encoder_fields;
                Ok(())
            }
            fn logpack_sizer(&self) -> usize {
                #sizer_fields
            }
        }
    };

    result
}

fn encoder_for_struct(variant_data: &syn::VariantData, sized: bool) -> quote::Tokens
{
    match *variant_data {
        syn::VariantData::Struct(ref fields) => {
            encoder_for_struct_kind(Some(&fields), true, sized)
        },
        syn::VariantData::Tuple(ref fields) => {
            encoder_for_struct_kind(Some(&fields), false, sized)
        },
        syn::VariantData::Unit => {
            encoder_for_struct_kind(None, false, sized)
        },
    }
}

fn encoder_for_enum_struct(name: &syn::Ident, ident: &syn::Ident,
                           fields: Vec<FieldExt>, prefix: quote::Tokens,
                           named: bool, sizer: bool, header_size: usize) -> quote::Tokens {
    let one_ref = fields.iter().map(|v| {
        let ident = &v.match_ident;
        quote! { ref #ident }
    });

    let fields_match = match named {
        false => quote!(( #(#one_ref),* )),
        true => quote!({ #(#one_ref),* }),
    };

    let body = if sizer {
        let body_impls = fields.iter().map(|v| {
            let ident = &v.match_ident;
            quote! { size += #ident.logpack_sizer(); }
        });
        quote!(let mut size: usize = #header_size; #(#body_impls);*; size )
    } else {
        let body_impls = fields.iter().map(|v| {
            let ident = &v.match_ident;
            quote! { #ident.logpack_encode(_buf)? }
        });
        quote!(#(#body_impls);*; Ok(()) )
    };

    quote! {
        &#name::#ident #fields_match => {
            #prefix
            #body
        }
    }
}

fn encoder_for_enum(name: &syn::Ident, variants: &[syn::Variant], sizer: bool) -> quote::Tokens {
    if variants.len() == 0 {
        if sizer {
            quote!(0)
        } else {
            quote!()
        }
    } else {
        let mut idx : u32 = 0;
        let (idx_type, header_size) = if variants.len() < 0x100 {
            ("u8", 1)
        } else if variants.len() < 0x10000 {
            ("u16", 2)
        } else {
            ("u32", 4)
        };
        let idx_type = syn::Ident::new(idx_type);
        let impls = variants.iter().map(|v| {
            let ident = &v.ident;
            let prefix = if sizer {
                quote! {}
            } else {
                quote! {
                    let idx : #idx_type = #idx as #idx_type;
                    idx.logpack_encode(_buf)?;
                }
            };

            idx += 1;
            match v.data {
                syn::VariantData::Struct(ref fields) => {
                    let fields: Vec<_> = fields.iter().enumerate().map(|(i, f)|
                            FieldExt::new(f, i, true)).collect();
                    encoder_for_enum_struct(name, ident, fields, prefix, true,
                                            sizer, header_size)
                },
                syn::VariantData::Tuple(ref fields) => {
                    let fields: Vec<_> = fields.iter().enumerate().map(|(i, f)|
                            FieldExt::new(f, i, false)).collect();
                    encoder_for_enum_struct(name, ident, fields, prefix, false,
                                            sizer, header_size)
                },
                syn::VariantData::Unit => {
                    if sizer {
                        quote! { &#name::#ident => { #header_size } }
                    } else {
                        quote! {
                            &#name::#ident => {
                                #prefix
                                Ok(())
                            }
                        }
                    }
                },
            }
        });

        if sizer {
            quote!(
                match self {
                    #(#impls),*
                }
            )
        } else {
            quote!(
                match self {
                    #(#impls),*
                }?
            )
        }
    }
}

fn encoder_for_struct_kind(fields: Option<&[syn::Field]>, named: bool, sizer: bool) -> quote::Tokens
{
    let unit = fields.is_none();
    let fields: Vec<_> = fields.unwrap_or(&[]).iter()
        .enumerate().map(|(i, f)| FieldExt::new(f, i, named)).collect();
    if unit {
        if sizer {
            quote![ 0 ]
        } else {
            quote![ ]
        }
    } else {
        let fields = fields.iter().map(|f| {
            let f_name = &f.access_ident;
            if sizer {
                quote!(size += self.#f_name.logpack_sizer();)
            } else {
                quote!(self.#f_name.logpack_encode(_buf)?)
            }
        });
        if sizer {
            quote!{
                let mut size : usize = 0;
                #(#fields);*;
                size
            }
        } else {
            quote!{ #(#fields);* }
        }
    }
}

struct FieldExt {
    access_ident: syn::Ident,
    match_ident: syn::Ident,
}

impl FieldExt {
    pub fn new(field: &syn::Field, idx: usize, named: bool) -> FieldExt {
        FieldExt {
            access_ident: if named {
                field.ident.clone().unwrap()
            } else {
                syn::Ident::new(format!("{}", idx))
            },
            match_ident: if named {
                field.ident.clone().unwrap()
            } else {
                syn::Ident::new(format!("f{}", idx))
            },
        }
    }
}

