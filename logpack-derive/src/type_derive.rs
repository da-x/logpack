use proc_macro2::{TokenStream as Tokens};
use syn::{Data, DeriveInput, Fields, DataEnum};
use std::collections::HashSet;

pub fn derive(input: &DeriveInput) -> Tokens {
    let name = &input.ident;
    let generics = super::add_trait_bounds(
        input.generics.clone(),
        &HashSet::new(),
        &["logpack::Logpack"],
        &name,
    );
    let (_, ty_generics, where_clause) = generics.split_for_impl();

    let fields = match &input.data {
        Data::Enum(data) => {
            let fields = bintype_for_enum(data);
            quote! { Some(logpack::Named::Enum( #fields )) }
        }
        Data::Struct(data) => {
            let fields = bintype_for_struct(&data.fields);
            quote! { Some(logpack::Named::Struct( #fields )) }
        }
        Data::Union{..} => {
            panic!()
        }
    };

     let result = quote! {
         impl #ty_generics logpack::Logpack for #name #ty_generics #where_clause {
             fn logpack_describe(st: &mut logpack::SeenTypes) ->
                 logpack::Description<logpack::TypeNameId, logpack::FieldName>
             {
                 use std::any::TypeId;
                 let self_id = TypeId::of::<Self>();
                 let (first_seen, typename_id) = st.make_name_for_id(stringify!(#name), self_id);
                 let may_recurse = if first_seen { #fields } else { None };

                 logpack::Description::ByName(typename_id, may_recurse)
             }
         }
     };

     result
}

fn bintype_for_struct(fields: &Fields) -> Tokens {
    match fields {
        Fields::Named(ref fields) => {
            let fields : Vec<_> = fields.named.iter().map(|f| {
                let f_name = &f.ident;
                let ty = &f.ty;
                quote!((stringify!(#f_name),
                        logpack::LogpackWrapper::<#ty>::logpack_describe(st)))
            }).collect();
            quote![ logpack::Struct::Named(vec![ #(#fields),* ]) ]
        },
        Fields::Unnamed(ref fields) => {
            let fields : Vec<_> = fields.unnamed.iter().map(|f| {
                let ty = &f.ty;
                quote!(logpack::LogpackWrapper::<#ty>::logpack_describe(st))
            }).collect();
            quote![ logpack::Struct::Tuple(vec![ #(#fields),* ]) ]
        },
        Fields::Unit => {
            quote![ logpack::Struct::Unit ]
        },
    }
}

fn bintype_for_enum(data_enum: &DataEnum) -> Tokens {
    let impls = data_enum.variants.iter().map(|v| {
        let fields = bintype_for_struct(&v.fields);
        let ident = &v.ident;
        quote! { (stringify!(#ident), #fields) }
    });

    quote!(vec![#(#impls),*])
}
