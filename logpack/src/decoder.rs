use super::Description;
use super::Named;
use super::Struct;

use std::collections::{HashMap};
use buffers::BufDecoder;

pub type TypeName = String;
pub type TypeNameId = (TypeName, u16);

pub struct NameMap {
    map: HashMap<TypeNameId, Named<TypeNameId>>
}

#[derive(Debug)]
pub enum FeedError {
    Dups,
}

pub type FeedResult<T> = Result<T, FeedError>;

impl NameMap {
    pub fn new() -> Self {
        Self {
            map : HashMap::new(),
        }
    }

    pub fn feed(&mut self, description: Description<TypeNameId>) -> FeedResult<Description<TypeNameId>>
    {
        use Description::*;

        Ok(match description {
            Option(o) => Option(Box::new(self.feed(*o)?)),
            Slice(o) => Slice(Box::new(self.feed(*o)?)),
            Array(size, o) => Array(size, Box::new(self.feed(*o)?)),
            Result(t, f) => Result(Box::new(self.feed(*t)?), Box::new(self.feed(*f)?)),
            Tuple(vec) => Tuple({
                let items: ::std::result::Result<Vec<_>, _> = vec.into_iter().map(|x| self.feed(x)).collect();
                items?
            }),

            q@U64 => q,
            q@U32 => q,
            q@U16 => q,
            q@U8 => q,
            q@I64 => q,
            q@I32 => q,
            q@I16 => q,
            q@I8 => q,
            q@Unit => q,
            q@PhantomData => q,
            q@Bool => q,
            q@String => q,

            q@ByName(_, None) => q,
            ByName(name, Some(named)) => {
                let v = self.feed_named(named)?;
                self.map.insert(name.clone(), v);
                ByName(name, None)
            },
        })
    }

    pub fn feed_named(&mut self, named: Named<TypeNameId>) -> FeedResult<Named<TypeNameId>>
    {
        use Named::*;

        Ok(match named {
            Enum(vec) => Enum({
                let items: ::std::result::Result<Vec<_>, _> =
                    vec.into_iter().map(|(name, x)| Ok((name, self.feed_struct(x)?))).collect();
                items?
            }),
            Struct(struct_) => Struct(self.feed_struct(struct_)?),
        })
    }

    pub fn feed_struct(&mut self, struct_: Struct<TypeNameId>) -> FeedResult<Struct<TypeNameId>>
    {
        use Struct::*;

        Ok(match struct_ {
            Unit => Unit,
            Tuple(vec) => Tuple({
                let items: ::std::result::Result<Vec<_>, _> =
                    vec.into_iter().map(|x|self.feed(x)).collect();
                items?
            }),
            Named(vec) => Named({
                let items: ::std::result::Result<Vec<_>, _> =
                   vec.into_iter().map(|(name, x)| Ok((name, self.feed(x)?))).collect();
                items?
            }),
        })
    }
}

pub struct Decoder<'a, 'b>
{
    types: &'a NameMap,
    buffer: BufDecoder<'b>,
}

pub trait Callbacks {
    type SubType : Callbacks;

    fn handle_u8(&mut self, u8);
    fn handle_u16(&mut self, u16);
    fn handle_u32(&mut self, u32);
    fn handle_u64(&mut self, u64);
    fn handle_i8(&mut self, i32);
    fn handle_i16(&mut self, i32);
    fn handle_i32(&mut self, i32);
    fn handle_i64(&mut self, i64);
    fn handle_bool(&mut self, bool);
    fn handle_string(&mut self, &str);
    fn handle_unit(&mut self);
    fn handle_phantom(&mut self);

    fn begin_enum(&mut self, typename_id: &TypeNameId, option_name: &String) -> &mut Self::SubType;
    fn end_enum(&mut self, typename_id: &TypeNameId);

    fn option_none(&mut self);

    fn option_some(&mut self) -> &mut Self::SubType;
    fn option_end(&mut self);

    fn result_ok(&mut self) -> &mut Self::SubType;
    fn result_err(&mut self) -> &mut Self::SubType;
    fn result_end(&mut self);

    fn struct_unit(&mut self, typename_id: Option<&TypeNameId>);

    fn begin_struct_named(&mut self, typename_id: Option<&TypeNameId>) -> &mut Self::SubType;
    fn begin_named_field(&mut self, field_idx: u16, field_name: &String) -> &mut Self::SubType;
    fn end_named_field(&mut self);
    fn end_struct_named(&mut self);

    fn begin_struct_tuple(&mut self, typename_id: Option<&TypeNameId>) -> &mut Self::SubType;
    fn begin_tuple_field(&mut self, field_idx: u16) -> &mut Self::SubType;
    fn end_tuple_field(&mut self);
    fn end_struct_tuple(&mut self);

    fn begin_tuple(&mut self, size: usize) -> &mut Self::SubType;
    fn begin_tuple_item(&mut self, field_idx: u16);
    fn end_tuple_item(&mut self);
    fn end_tuple(&mut self);

    fn begin_array(&mut self, size: usize) -> &mut Self::SubType;
    fn begin_array_item(&mut self, field_idx: u16);
    fn end_array_item(&mut self);
    fn end_array(&mut self);

    fn begin_slice(&mut self, size: usize) -> &mut Self::SubType;
    fn begin_slice_item(&mut self, field_idx: u16);
    fn end_slice_item(&mut self);
    fn end_slice(&mut self);
}

use std::str::{Utf8Error, self};

#[derive(Debug)]
pub enum Error {
    MissingType(TypeNameId),
    UTF8Error(Utf8Error),
    GetError((usize, usize)),
    InvalidIndex(usize, usize),
    InvalidSome(u8),
    InvalidResult(u8),
}

macro_rules! simple {
    ($self:ident, $callbacks:ident, $func:ident) => {
            {
                let val = $self.buffer.get::<_>().map_err(Error::GetError)?;
                $callbacks.$func(val);
                Ok(())
            }
    }
}

impl<'a, 'b> Decoder<'a, 'b> {
    pub fn new(types: &'a NameMap, buffer: BufDecoder<'b>) -> Self {
        Self { types, buffer }
    }

    pub fn decode<C>(&mut self, desc: &Description<TypeNameId>, callbacks: &mut C) -> Result<(), Error>
        where C: Callbacks
    {
        use Description::*;
        match desc {
            &U8 => simple!(self, callbacks, handle_u8),
            &U16 => simple!(self, callbacks, handle_u16),
            &U32 => simple!(self, callbacks, handle_u32),
            &U64 => simple!(self, callbacks, handle_u64),
            &I8 => simple!(self, callbacks, handle_i8),
            &I16 => simple!(self, callbacks, handle_i16),
            &I32 => simple!(self, callbacks, handle_i32),
            &I64 => simple!(self, callbacks, handle_i64),
            &Bool => simple!(self, callbacks, handle_bool),
            &Unit => { callbacks.handle_unit(); Ok(()) }
            &PhantomData => { callbacks.handle_phantom(); Ok(()) }
            &ByName(ref typename_id, None) => {
                self.decode_by_name(typename_id, callbacks)
            }
            &ByName(ref typename_id, Some(ref desc)) => {
                self.decode_by_name_direct(typename_id, desc, callbacks)
            }
            &String => self.decode_string(callbacks),
            &Option(ref sub) => self.decode_option(sub, callbacks),
            &Result(ref sub, ref sub2) => self.decode_result(sub, sub2, callbacks),
            &Array(size, ref sub) => self.decode_array(size, sub, callbacks),
            &Slice(ref sub) => self.decode_slice(sub, callbacks),
            &Tuple(ref subs) => self.decode_tuple(subs, callbacks),
        }
    }

    pub fn decode_by_name<C>(&mut self, typename_id: &TypeNameId, callbacks: &mut C) -> Result<(), Error>
        where C: Callbacks
    {
        let named = match self.types.map.get(typename_id) {
            None => return Err(Error::MissingType(typename_id.clone())),
            Some(x) => x,
        };

        self.decode_by_name_direct(typename_id, named, callbacks)
    }

    fn decode_array<C>(&mut self, size: usize, sub: &Box<Description<TypeNameId>>, callbacks: &mut C) -> Result<(), Error>
        where C: Callbacks
    {
        let ctx = callbacks.begin_array(size);
        let mut idx = 0;
        for _ in 0 .. size {
            ctx.begin_array_item(idx);
            self.decode(sub, ctx)?;
            ctx.end_array_item();
            idx += 1;
        }
        ctx.end_array();
        Ok(())
    }

    fn decode_slice<C>(&mut self, sub: &Box<Description<TypeNameId>>, callbacks: &mut C) -> Result<(), Error>
        where C: Callbacks
    {
        let size = self.buffer.get::<u64>().map_err(Error::GetError)? as usize;
        let ctx = callbacks.begin_slice(size);
        let mut idx = 0;
        for _ in 0 .. size {
            ctx.begin_slice_item(idx);
            self.decode(sub, ctx)?;
            ctx.end_slice_item();
            idx += 1;
        }
        ctx.end_slice();
        Ok(())
    }

    fn decode_tuple<C>(&mut self, subs: &Vec<Description<TypeNameId>>, callbacks: &mut C) -> Result<(), Error>
        where C: Callbacks
    {
        let ctx = callbacks.begin_tuple(subs.len());
        let mut idx = 0;
        for v in subs.iter() {
            ctx.begin_tuple_item(idx);
            self.decode(v, ctx)?;
            ctx.end_tuple_item();
            idx += 1;
        }
        ctx.end_tuple();
        Ok(())
    }

    fn decode_option<C>(&mut self, desc: &Description<TypeNameId>, callbacks: &mut C) -> Result<(), Error>
        where C: Callbacks
    {
        let f0 = self.buffer.get::<u8>().map_err(Error::GetError)?;

        match f0 {
            0 => callbacks.option_none(),
            1 => {
                let ctx = callbacks.option_some();
                self.decode(desc, ctx)?;
                ctx.option_end();
            },
            n => return Err(Error::InvalidSome(n)),
        }

        Ok(())
    }

    fn decode_result<C>(&mut self, desc: &Description<TypeNameId>, desc2: &Description<TypeNameId>, callbacks: &mut C) -> Result<(), Error>
        where C: Callbacks
    {
        let f0 = self.buffer.get::<u8>().map_err(Error::GetError)?;

        match f0 {
            0 => {
                let ctx = callbacks.result_ok();
                self.decode(desc, ctx)?;
                ctx.result_end();
            },
            1 => {
                let ctx = callbacks.result_err();
                self.decode(desc2, ctx)?;
                ctx.result_end();
            },
            n => return Err(Error::InvalidResult(n)),
        }

        Ok(())
    }

    fn decode_string<C>(&mut self, callbacks: &mut C) -> Result<(), Error>
        where C: Callbacks
    {
        let f0 = self.buffer.get::<u8>().map_err(Error::GetError)?;

        let extra_header = f0 & 0x3;
        let len = match extra_header {
            0 => (f0 >> 2) as u64,
            1 => {
                let f1 = self.buffer.get::<u8>().map_err(Error::GetError)?;
                ((f1 as u64) << 6) | ((f0 >> 2) as u64)
            }
            2 => {
                let f1 = self.buffer.get::<u8>().map_err(Error::GetError)?;
                let f2 = self.buffer.get::<u16>().map_err(Error::GetError)?;
                ((f2 as u64) << 14) |((f1 as u64) << 6) | ((f0 >> 2) as u64)
            }
            3 => {
                let f1 = self.buffer.get::<u8>().map_err(Error::GetError)?;
                let f2 = self.buffer.get::<u16>().map_err(Error::GetError)?;
                let f3 = self.buffer.get::<u32>().map_err(Error::GetError)?;

                ((f3 as u64) << 30) | ((f2 as u64) << 14) | ((f1 as u64) << 6) | ((f0 >> 2) as u64)
            }
            _ => panic!(),
        };

        let u8slice = self.buffer.get_slice(len as usize).map_err(Error::GetError)?;
        let strslice = str::from_utf8(u8slice).map_err(Error::UTF8Error)?;
        callbacks.handle_string(strslice);

        Ok(())
    }

    fn decode_by_name_direct<C>(&mut self, typename_id: &TypeNameId, named: &Named<TypeNameId>, callbacks: &mut C) -> Result<(), Error>
        where C: Callbacks
    {
        use Named::*;

        match named {
            &Enum(ref optvec) => {
                let len = optvec.len();
                let idx = if len < 0x100 {
                    self.buffer.get::<u8>().map_err(Error::GetError)? as usize
                } else if len < 0x10000 {
                    self.buffer.get::<u16>().map_err(Error::GetError)? as usize
                } else {
                    self.buffer.get::<u32>().map_err(Error::GetError)? as usize
                };
                if idx >= len {
                    return Err(Error::InvalidIndex(idx, len));
                }
                let ctx = callbacks.begin_enum(typename_id, &optvec[idx].0);
                self.decode_struct(None, &optvec[idx].1, ctx)?;
                ctx.end_enum(typename_id);
            }
            &Struct(ref desc) => {
                self.decode_struct(Some(typename_id), desc, callbacks)?;
            }
        }

        Ok(())
    }

    fn decode_struct<C>(&mut self, typename_id: Option<&TypeNameId>, struct_desc: &Struct<TypeNameId>, callbacks: &mut C) -> Result<(), Error>
        where C: Callbacks
    {
        use Struct::*;

        match struct_desc {
            &Unit => {
                callbacks.struct_unit(typename_id);
            }
            &Named(ref v) => {
                let ctx = callbacks.begin_struct_named(typename_id);
                let mut idx = 0;
                for &(ref key, ref value) in v.iter() {
                    let ctx = ctx.begin_named_field( idx, key);
                    self.decode(value, ctx)?;
                    ctx.end_named_field();
                    idx += 1;
                }
                ctx.end_struct_named();
            }
            &Tuple(ref v) => {
                let ctx = callbacks.begin_struct_tuple(typename_id);
                let mut idx = 0;
                for ref value in v.iter() {
                    let ctx = ctx.begin_tuple_field(idx);
                    self.decode(value, ctx)?;
                    ctx.end_tuple_field();
                    idx += 1;
                }
                ctx.end_struct_tuple();
            }
        }

        Ok(())
    }
}
