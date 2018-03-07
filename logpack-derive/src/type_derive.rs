use proc_macro::TokenStream;
use syn;
use quote;

// TODO: use the likewise of serde-derive's with_bound so that 'T: LogpackType' constraints
// will not be needed at struct definition.

pub fn derive(input: &TokenStream) -> quote::Tokens {
    let input: String = input.clone().to_string();
    let ast = syn::parse_macro_input(&input).expect("Couldn't parse item");
    let name = &ast.ident;
    let (_, ty_generics, where_clause) = ast.generics.split_for_impl();

    let fields = match ast.body {
        syn::Body::Enum(ref variants) => {
            let fields = bintype_for_enum(&variants);
            quote! { Some(::logpack::Named::Enum( #fields )) }
        }
        syn::Body::Struct(ref variant_data) => {
            let fields = bintype_for_struct(&variant_data);
            quote! { Some(::logpack::Named::Struct( #fields )) }
        }
    };

    let result = quote! {
        impl #ty_generics ::logpack::LogpackType for #name #ty_generics #where_clause {
            fn logpack_describe(st: &mut ::logpack::SeenTypes) ->
                ::logpack::Description<::logpack::TypeNameId, ::logpack::FieldName>
            {
                use std::any::TypeId;
                let self_id = TypeId::of::<Self>();
                let (first_seen, typename_id) = st.make_name_for_id(stringify!(#name), self_id);
                let may_recurse = if first_seen { #fields } else { None };

                ::logpack::Description::ByName(typename_id, may_recurse)
            }
        }
    };

    result
}

fn bintype_for_struct(variant_data: &syn::VariantData) -> quote::Tokens
{
    match *variant_data {
        syn::VariantData::Struct(ref fields) => {
            bintype_for_struct_kind(Some(&fields), true)
        },
        syn::VariantData::Unit => {
            bintype_for_struct_kind(None, false)
        },
        syn::VariantData::Tuple(ref fields) => {
            bintype_for_struct_kind(Some(&fields), false)
        },
    }
}

fn bintype_for_enum(variants: &[syn::Variant]) -> quote::Tokens {
    let impls = variants.iter().map(|v| {
        let fields = bintype_for_struct(&v.data);
        let ident = &v.ident;
        quote! { (stringify!(#ident), #fields) }
    });

    quote!(vec![#(#impls),*])
}

fn bintype_for_struct_kind(fields: Option<&[syn::Field]>, named: bool) -> quote::Tokens
{
    let unit = fields.is_none();
    let fields: Vec<_> = fields.unwrap_or(&[]).iter()
        .enumerate().map(|(i, f)| FieldExt::new(f, i, named)).collect();
    if unit {
        quote![ ::logpack::Struct::Unit ]
    } else if named {
        let fields = fields.iter().map(|f| f.as_named());
        quote![ ::logpack::Struct::Named(vec![ #(#fields),* ]) ]
    } else {
        let fields = fields.iter().map(|f| f.as_tuple());
        quote![ ::logpack::Struct::Tuple(vec![ #(#fields),* ]) ]
    }
}

struct FieldExt<'a> {
    ty: &'a syn::Ty,
    ident: syn::Ident,
}

impl<'a> FieldExt<'a> {
    pub fn new(field: &'a syn::Field, idx: usize, named: bool) -> FieldExt<'a> {
        FieldExt {
            ty: &field.ty,
            ident: if named {
                field.ident.clone().unwrap()
            } else {
                syn::Ident::new(format!("f{}", idx))
            },
        }
    }

    pub fn as_named(&self) -> quote::Tokens {
        let f_name = &self.ident;
        let ty = &self.ty;
        quote!((stringify!(#f_name),
                ::logpack::LogpackTypeWrapper::<#ty>::logpack_describe(st)))
    }

    pub fn as_tuple(&self) -> quote::Tokens {
        let ty = &self.ty;
        quote!(::logpack::LogpackTypeWrapper::<#ty>::logpack_describe(st))
    }
}
