use serde_derive::{Serialize, Deserialize};

pub mod decoder;
pub mod encoder;
pub mod buffers;

pub use encoder::Encoder;
pub use decoder::Decoder;
pub use decoder::NameMap;
pub use buffers::BufEncoder;
pub use buffers::BufDecoder;
pub use decoder::ResolvedDesc;

use std::collections::HashMap;
use std::marker::PhantomData;
use std::any::TypeId;

//////////////////////////////////////////////////////////////////////////
//
// Type description

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub enum Struct<T, S=String> {
    Unit,
    Tuple(Vec<Description<T, S>>),
    Named(Vec<(S, Description<T, S>)>),
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub enum Named<T, S=String> {
    Enum(Vec<(S, Struct<T, S>)>),
    Struct(Struct<T, S>),
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub enum Description<T, S=String> {
    U64,
    U32,
    U16,
    U8,
    I64,
    I32,
    I16,
    I8,
    Unit,
    PhantomData,
    Bool,
    String,
    RawPtr,

    Option(Box<Description<T, S>>),
    Result(Box<Description<T, S>>, Box<Description<T, S>>),

    Array(usize, Box<Description<T, S>>),
    Slice(Box<Description<T, S>>),
    Tuple(Vec<Description<T, S>>),

    ByName(T, Option<Named<T, S>>),
}

//////////////////////////////////////////////////////////////////////////
//
// SeenTypes

pub type TypeName = &'static str;
pub type FieldName = &'static str;
pub type TypeNameId = (TypeName, u16);

pub struct SeenTypes {
    by_ids: HashMap<TypeId, (TypeName, u16)>,
    names: HashMap<TypeName, u16>,
}

impl SeenTypes {
    pub fn new() -> Self {
        Self {
            by_ids: HashMap::new(),
            names: HashMap::new(),
        }
    }

    pub fn make_name_for_id(&mut self, name: &'static str, type_id: TypeId) -> (bool, TypeNameId) {
        if let Some(value) = self.by_ids.get(&type_id) {
            return (false, *value);
        }

        if let Some(value) = self.names.get_mut(name) {
            *value += 1;
            let v = (name, *value);
            self.by_ids.insert(type_id, v);
            return (true, v);
        }

        let v = (name, 0);
        self.names.insert(name, 0);
        self.by_ids.insert(type_id, v);
        (true, v)
    }
}

//////////////////////////////////////////////////////////////////////////
//
// Logpack and impl

pub type RefDesc = Description<TypeNameId, FieldName>;

pub trait Logpack {
    fn logpack_describe(seen: &mut SeenTypes) -> RefDesc;
    fn logpack_describe_by_value(&self, seen: &mut SeenTypes) -> RefDesc {
        Self::logpack_describe(seen)
    }
}

macro_rules! simple {
    ($a:tt, $b:ident) => {
        impl Logpack for $a {
            fn logpack_describe(_: &mut SeenTypes) -> RefDesc {
                Description::$b
            }
        }
    }
}

simple!(usize, U64);
simple!(u64, U64);
simple!(u32, U32);
simple!(u16, U16);
simple!(u8, U8);
simple!(isize, I64);
simple!(i64, I64);
simple!(i32, I32);
simple!(i16, I16);
simple!(i8, I8);
simple!((), Unit);
simple!(bool, Bool);
simple!(str, String);
simple!(String, String);

impl<T> Logpack for Option<T>
    where T: Logpack
{
    fn logpack_describe(seen: &mut SeenTypes) -> RefDesc{
        Description::Option(Box::new(T::logpack_describe(seen)))
    }
}

impl<T, S> Logpack for Result<T, S>
    where T: Logpack, S: Logpack
{
    fn logpack_describe(seen: &mut SeenTypes) -> RefDesc {
        Description::Result(Box::new(T::logpack_describe(seen)),
                            Box::new(S::logpack_describe(seen)))
    }
}

impl<T> Logpack for PhantomData<T>
    where T: Logpack
{
    fn logpack_describe(_: &mut SeenTypes) -> RefDesc {
        Description::PhantomData
    }
}

impl<T> Logpack for [T; 0] {
    fn logpack_describe(_: &mut SeenTypes) -> RefDesc {
        Description::Unit
    }
}

macro_rules! array_impls {
    ($($len:tt)+) => {
        $(
            impl<T> Logpack for [T; $len]
                where T: Logpack,
            {
                fn logpack_describe(seen: &mut SeenTypes) -> RefDesc {
                    Description::Array($len, Box::new(T::logpack_describe(seen)))
                }
            }
        )+
    }
}

array_impls!(01 02 03 04 05 06 07 08 09 10
             11 12 13 14 15 16 17 18 19 20
             21 22 23 24 25 26 27 28 29 30
             31 32);

macro_rules! tuple {
    ($($type:ident),*) => {
        impl<$($type),*> Logpack for ($($type),*)
            where $($type : Logpack),*
        {
            fn logpack_describe(seen: &mut SeenTypes) -> RefDesc {
                Description::Tuple(vec![
                    $($type::logpack_describe(seen)),*
            ]) }
        }
    }
}

tuple!(A, B);
tuple!(A, B, C);
tuple!(A, B, C, D);
tuple!(A, B, C, D, E);
tuple!(A, B, C, D, E, F);
tuple!(A, B, C, D, E, F, G);
tuple!(A, B, C, D, E, F, G, H);
tuple!(A, B, C, D, E, F, G, H, I);
tuple!(A, B, C, D, E, F, G, H, I, J);
tuple!(A, B, C, D, E, F, G, H, I, J, K);
tuple!(A, B, C, D, E, F, G, H, I, J, K, L);
tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M);
tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N);
tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O);
tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, R);
tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, R, P);

impl<T> Logpack for [T] where T: Logpack
{
    fn logpack_describe(seen: &mut SeenTypes) -> RefDesc {
        Description::Slice(Box::new(T::logpack_describe(seen)))
    }
}

macro_rules! deref_impl {
    ($($desc:tt)+) => {
        impl $($desc)+ {
            #[inline]
            fn logpack_describe(seen: &mut SeenTypes) -> RefDesc {
                T::logpack_describe(seen)
            }
        }
    };
}

deref_impl!(<'a, T: ?Sized> Logpack for &'a T where T: Logpack);
deref_impl!(<'a, T: ?Sized> Logpack for &'a mut T where T: Logpack);

impl<T> Logpack for Box<T> where T: Logpack
{
    fn logpack_describe(seen: &mut SeenTypes) -> RefDesc {
        T::logpack_describe(seen)
    }
}

pub struct LogpackWrapper<T>(T);

impl<T> Logpack for LogpackWrapper<T> where T: Logpack
{
    fn logpack_describe(seen: &mut SeenTypes) -> RefDesc {
        T::logpack_describe(seen)
    }
}

impl<T> Logpack for *const T
{
    fn logpack_describe(_seen: &mut SeenTypes) -> RefDesc{
        Description::RawPtr
    }
}

impl<T> Logpack for *mut T
{
    fn logpack_describe(_seen: &mut SeenTypes) -> RefDesc{
        Description::RawPtr
    }
}

//////////////////////////////////////////////////////////////////////////

macro_rules! std_type_to_tuple {
    ($name:ident: $($fields:ident),+) => {
        impl Logpack for $name
        {
            fn logpack_describe(seen: &mut SeenTypes) -> RefDesc {
                let (first_seen, typename_id) = seen.make_name_for_id(stringify!($name),
                                                                      TypeId::of::<Self>());
                let may_recurse = if first_seen {
                    Some(Named::Struct(Struct::Tuple(vec![
                        $( $fields::logpack_describe(seen) ),*
                    ])))
                } else {
                    None
                };

                Description::ByName(typename_id, may_recurse)
            }
        }
    };
}

use std::time::Duration;
use std::time::Instant;

std_type_to_tuple!(Duration: u64, u32);
std_type_to_tuple!(Instant: u64, u32);
