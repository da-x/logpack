use std::mem::size_of;

use super::buffers;

pub trait Encoder {
    fn logpack_encode(&self, buf: &mut buffers::BufEncoder) -> Result<(), (usize, usize)>;
    fn logpack_sizer(&self) -> usize;
}

macro_rules! simple {
    ($a:tt) => {
        impl Encoder for $a {
            #[inline(always)]
            fn logpack_encode(&self, buf: &mut buffers::BufEncoder) -> Result<(), (usize, usize)> {
                buf.put(self)
            }
            #[inline(always)]
            fn logpack_sizer(&self) -> usize {
                size_of::<Self>()
            }
        }
    }
}

simple!(usize);
simple!(u64);
simple!(u32);
simple!(u16);
simple!(u8);
simple!(isize);
simple!(i64);
simple!(i32);
simple!(i16);
simple!(i8);
simple!(bool);

impl Encoder for () {
    #[inline(always)]
    fn logpack_encode(&self, _buf: &mut buffers::BufEncoder) -> Result<(), (usize, usize)> {
        Ok(())
    }
    #[inline(always)]
    fn logpack_sizer(&self) -> usize { 0 }
}

pub fn encoded_string_len(value: &str) -> usize
{
    let bytes = value.as_bytes();
    let size = bytes.len();

    if size < 0x40 { return 1 + size };
    if size < 0x4000 { return 2 + size };
    if size < 0x4000_0000 { return 4 + size };
    if size < 0x4000_0000_0000_0000 { return 8 + size };

    panic!("string length {}", size);
}

pub fn encode_stored_string(value: &str, buf: &mut buffers::BufEncoder)
    -> Result<(), (usize, usize)>
{
    let bytes = value.as_bytes();
    let size = bytes.len();

    // TODO: fix little-endian assumption

    if size < 0x40 {
        (0u8  | ((size as u8) << 2) ).logpack_encode(buf)?;
    } else if size < 0x4000 {
        (1u16 | ((size as u16) << 2) ).logpack_encode(buf)?;
    } else if size < 0x4000_0000 {
        (2u32 | ((size as u32) << 2) ).logpack_encode(buf)?;
    } else if size < 0x4000_0000_0000_0000 {
        (3u64 | ((size as u64) << 2) ).logpack_encode(buf)?;
    } else {
        panic!("string length {}", size);
    }

    unsafe {
        let space = buf.reserve_space_by_size(size)?;
        ::std::ptr::copy_nonoverlapping(bytes.as_ptr(), space, size);
    }
    Ok(())
}

impl<'a> Encoder for &'a str {
    #[inline(always)]
    fn logpack_encode(&self, buf: &mut buffers::BufEncoder) -> Result<(), (usize, usize)> {
        encode_stored_string(self, buf)
    }
    #[inline(always)]
    fn logpack_sizer(&self) -> usize {
        encoded_string_len(self)
    }
}

impl Encoder for String {
    #[inline(always)]
    fn logpack_encode(&self, buf: &mut buffers::BufEncoder) -> Result<(), (usize, usize)> {
        encode_stored_string(self.as_str(), buf)
    }
    #[inline(always)]
    fn logpack_sizer(&self) -> usize {
        encoded_string_len(self.as_str())
    }
}

macro_rules! array_impls {
    ($($len:tt)+) => {
        $(
            impl<T> Encoder for [T; $len]
                where T: Encoder,
            {
                fn logpack_encode(&self, buf: &mut buffers::BufEncoder) -> Result<(), (usize, usize)> {
                    for i in 0..$len {
                        self[i].logpack_encode(buf)?
                    }
                    Ok(())
                }
                fn logpack_sizer(&self) -> usize {
                    let mut size = 0;
                    for i in 0..$len {
                        size += self[i].logpack_sizer();
                    }
                    size
                }
            }
        )+
    }
}

array_impls!(00
             01 02 03 04 05 06 07 08 09 10
             11 12 13 14 15 16 17 18 19 20
             21 22 23 24 25 26 27 28 29 30
             31 32);

macro_rules! tuple {
    ($(($type:ident, $num:tt)),*) => {
        impl<$($type),*> Encoder for ($($type),*)
            where $($type : Encoder),*
        {
            fn logpack_encode(&self, buf: &mut buffers::BufEncoder) -> Result<(), (usize, usize)> {
                $(
                    $type::logpack_encode(&self.$num, buf)?;
                )*

                Ok(())
            }
            fn logpack_sizer(&self) -> usize {
                let mut size = 0;
                $( size += $type::logpack_sizer(&self.$num); )*
                size
            }
        }
    }
}

tuple!((A, 0), (B, 1));
tuple!((A, 0), (B, 1), (C, 2));
tuple!((A, 0), (B, 1), (C, 2), (D, 3));
tuple!((A, 0), (B, 1), (C, 2), (D, 3), (E, 4));
tuple!((A, 0), (B, 1), (C, 2), (D, 3), (E, 4), (F, 5));
tuple!((A, 0), (B, 1), (C, 2), (D, 3), (E, 4), (F, 5), (G, 6));
tuple!((A, 0), (B, 1), (C, 2), (D, 3), (E, 4), (F, 5), (G, 6), (H, 7));
tuple!((A, 0), (B, 1), (C, 2), (D, 3), (E, 4), (F, 5), (G, 6), (H, 7), (I, 8));
tuple!((A, 0), (B, 1), (C, 2), (D, 3), (E, 4), (F, 5), (G, 6), (H, 7), (I, 8), (J, 9));
tuple!((A, 0), (B, 1), (C, 2), (D, 3), (E, 4), (F, 5), (G, 6), (H, 7), (I, 8), (J, 9), (K, 10));
tuple!((A, 0), (B, 1), (C, 2), (D, 3), (E, 4), (F, 5), (G, 6), (H, 7), (I, 8), (J, 9), (K, 10), (L, 11));
tuple!((A, 0), (B, 1), (C, 2), (D, 3), (E, 4), (F, 5), (G, 6), (H, 7), (I, 8), (J, 9), (K, 10), (L, 11), (M, 12));
tuple!((A, 0), (B, 1), (C, 2), (D, 3), (E, 4), (F, 5), (G, 6), (H, 7), (I, 8), (J, 9), (K, 10), (L, 11), (M, 12), (N, 13));
tuple!((A, 0), (B, 1), (C, 2), (D, 3), (E, 4), (F, 5), (G, 6), (H, 7), (I, 8), (J, 9), (K, 10), (L, 11), (M, 12), (N, 13), (O, 14));
tuple!((A, 0), (B, 1), (C, 2), (D, 3), (E, 4), (F, 5), (G, 6), (H, 7), (I, 8), (J, 9), (K, 10), (L, 11), (M, 12), (N, 13), (O, 14), (P, 15));

impl<T> Encoder for [T]
    where T: Encoder
{
    #[inline(always)]
    fn logpack_encode(&self, buf: &mut buffers::BufEncoder) -> Result<(), (usize, usize)> {
        let size : u64 = self.len() as u64;
        size.logpack_encode(buf)?;

        for i in 0..size {
            self[i as usize].logpack_encode(buf)?;
        }

        Ok(())
    }

    #[inline(always)]
    fn logpack_sizer(&self) -> usize {
        let mut size = 0;
        for i in 0..size {
            size += self[i as usize].logpack_sizer();
        }
        size
    }
}


impl<T> Encoder for Box<T>
    where T: Encoder
{
    #[inline(always)]
    fn logpack_encode(&self, buf: &mut buffers::BufEncoder) -> Result<(), (usize, usize)> {
        (**self).logpack_encode(buf)
    }

    #[inline(always)]
    fn logpack_sizer(&self) -> usize {
        (**self).logpack_sizer()
    }
}

impl<T> Encoder for *mut T
{
    #[inline(always)]
    fn logpack_encode(&self, buf: &mut buffers::BufEncoder) -> Result<(), (usize, usize)> {
        (*self as u64).logpack_encode(buf)
    }

    #[inline(always)]
    fn logpack_sizer(&self) -> usize {
        (*self as u64).logpack_sizer()
    }
}

impl<T> Encoder for *const T
{
    #[inline(always)]
    fn logpack_encode(&self, buf: &mut buffers::BufEncoder) -> Result<(), (usize, usize)> {
        (*self as u64).logpack_encode(buf)
    }

    #[inline(always)]
    fn logpack_sizer(&self) -> usize {
        (*self as u64).logpack_sizer()
    }
}

impl<T> Encoder for Option<T>
    where T: Encoder
{
    #[inline(always)]
    fn logpack_encode(&self, buf: &mut buffers::BufEncoder) -> Result<(), (usize, usize)> {
        match self {
            &None => {
                (0u8).logpack_encode(buf)
            }
            &Some(ref val) => {
                (1u8).logpack_encode(buf)?;
                val.logpack_encode(buf)
            }
        }
    }
    #[inline(always)]
    fn logpack_sizer(&self) -> usize {
        match self {
            &None => 1,
            &Some(ref val) => {
                1 + val.logpack_sizer()
            }
        }
    }
}


impl<T, E> Encoder for Result<T, E>
    where T: Encoder, E: Encoder
{
    #[inline(always)]
    fn logpack_encode(&self, buf: &mut buffers::BufEncoder) -> Result<(), (usize, usize)> {
        match self {
            &Ok(ref val) => {
               (0u8).logpack_encode(buf)?;
                val.logpack_encode(buf)
            }
            &Err(ref val) => {
                (1u8).logpack_encode(buf)?;
                val.logpack_encode(buf)
            }
        }
    }

    #[inline(always)]
    fn logpack_sizer(&self) -> usize {
        match self {
            &Ok(ref val) => {
                1 + val.logpack_sizer()
            }
            &Err(ref val) => {
                1 + val.logpack_sizer()
            }
        }
    }
}

//////////////////////////////////////////////////////////////////////

use std::time::Duration;

impl Encoder for Duration
{
    #[inline(always)]
    fn logpack_encode(&self, buf: &mut buffers::BufEncoder) -> Result<(), (usize, usize)> {
        let secs: u64 = self.as_secs();
        secs.logpack_encode(buf)?;
        let nanos: u32 = self.subsec_nanos();
        nanos.logpack_encode(buf)?;
        Ok(())
    }

    #[inline(always)]
    fn logpack_sizer(&self) -> usize {
        let secs: u64 = 0;
        let nanos: u32 = 0;
        secs.logpack_sizer() + nanos.logpack_sizer()
    }
}

cfg_if! {
    if #[cfg(unix)] {
        use std::time::Instant;

        #[cfg(not(any(target_os = "macos", target_os = "ios")))]
        impl Encoder for Instant
        {
            #[inline(always)]
            fn logpack_encode(&self, buf: &mut buffers::BufEncoder) -> Result<(), (usize, usize)> {
                use libc::timespec;
                let timespec = unsafe {
                    ::std::mem::transmute::<_, &timespec>(&self)
                };
                let secs: u64 = timespec.tv_sec as u64;
                secs.logpack_encode(buf)?;
                let nanos: u32 = timespec.tv_nsec as u32;
                nanos.logpack_encode(buf)?;
                Ok(())
            }

            #[inline(always)]
            fn logpack_sizer(&self) -> usize {
                let secs: u64 = 0;
                let nanos: u32 = 0;
                secs.logpack_sizer() + nanos.logpack_sizer()
            }
        }
    }
}
