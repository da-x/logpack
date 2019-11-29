use proc_macro2::{TokenStream as Tokens, Span};
use syn::{Data, DeriveInput, Fields, DataEnum, Ident};
use std::collections::HashSet;

pub fn derive(input: &DeriveInput) -> Tokens {
    let name = &input.ident;
    let generics = super::add_trait_bounds(
        input.generics.clone(),
        &HashSet::new(),
        &["logpack::Encoder"],
        &name,
    );
    let (_, ty_generics, where_clause) = generics.split_for_impl();

    let encoder_fields = match &input.data {
        Data::Enum(data) => encoder_for_enum(name, data, false),
        Data::Struct(variant_data) => encoder_for_struct(&variant_data.fields, false),
        Data::Union{..} => { panic!() }
    };

    let sizer_fields = match &input.data {
        Data::Enum(data) => encoder_for_enum(name, &data, true),
        Data::Struct(variant_data) => encoder_for_struct(&variant_data.fields, true),
        Data::Union{..} => { panic!() }
    };

    let result = quote! {
        impl #ty_generics logpack::Encoder for #name #ty_generics #where_clause {
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

fn encoder_for_struct(fields: &Fields, sized: bool) -> Tokens {
    match fields {
        Fields::Named(ref fields) => {
            let fields : Vec<_> = fields.named.iter().collect();

            encoder_for_struct_kind(Some(&fields[..]), true, sized)
        },
        Fields::Unnamed(ref fields) => {
            let fields : Vec<_> = fields.unnamed.iter().collect();

            encoder_for_struct_kind(Some(&fields[..]), false, sized)
        },
        Fields::Unit => {
            encoder_for_struct_kind(None, false, sized)
        },
    }
}

fn encoder_for_enum_struct<'a>(name: &Ident, ident: &Ident,
                               fields: Vec<FieldExt<'a>>, prefix: Tokens,
                               named: bool, sizer: bool, header_size: usize) -> Tokens {
    let one_ref = fields.iter().map(|v| {
        let ident = &v.get_match_ident();
        quote! { ref #ident }
    });

    let fields_match = match named {
        false => quote!(( #(#one_ref),* )),
        true => quote!({ #(#one_ref),* }),
    };

    let body = if sizer {
        let body_impls = fields.iter().map(|v| {
            let ident = &v.get_match_ident();
            quote! { size += #ident.logpack_sizer(); }
        });
        quote!(let mut size: usize = #header_size; #(#body_impls);*; size )
    } else {
        let body_impls = fields.iter().map(|v| {
            let ident = &v.get_match_ident();
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

fn encoder_for_enum(name: &Ident, data_enum: &DataEnum, sizer: bool) -> Tokens {
    let variants = &data_enum.variants;
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

        let idx_type = Ident::new(idx_type, Span::call_site());
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
            match v.fields {
                Fields::Named(ref fields) => {
                    let fields: Vec<_> = fields.named.iter().enumerate().map(|(i, f)|
                            FieldExt::new(f, i, true)).collect();
                    encoder_for_enum_struct(name, ident, fields, prefix, true,
                                            sizer, header_size)
                },
                Fields::Unnamed(ref fields) => {
                    let fields: Vec<_> = fields.unnamed.iter().enumerate().map(|(i, f)|
                            FieldExt::new(f, i, false)).collect();
                    encoder_for_enum_struct(name, ident, fields, prefix, false,
                                            sizer, header_size)
                },
                Fields::Unit => {
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

fn encoder_for_struct_kind(fields: Option<&[&syn::Field]>, named: bool, sizer: bool) -> Tokens {
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
            let field_expr = &f.access_expr();
            if sizer {
                quote!(size += #field_expr.logpack_sizer();)
            } else {
                quote!(#field_expr.logpack_encode(_buf)?)
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

struct FieldExt<'a> {
    field: &'a syn::Field,
    idx: usize,
    named: bool,
}

impl<'a> FieldExt<'a> {
    fn new(field: &'a syn::Field, idx: usize, named: bool) -> FieldExt<'a> {
        FieldExt { field, idx, named }
    }

    fn access_expr(&self) -> Tokens {
        if self.named {
            let ident = &self.field.ident;
            quote! { self.#ident }
        } else {
            let idx = syn::Index::from(self.idx);
            quote! { self.#idx }
        }
    }

    fn get_match_ident(&self) -> Ident {
        if self.named {
            self.field.ident.clone().unwrap()
        } else {
            Ident::new(&format!("f{}", self.idx), Span::call_site())
        }
    }
}

