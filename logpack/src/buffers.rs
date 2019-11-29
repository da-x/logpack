use std::mem::size_of;

//////////////////////////////////////////////////////////////////////////
// BufEncoder
//

pub struct BufEncoder<'a> {
    slice: &'a mut [u8],
    position: usize,
}

impl<'a> BufEncoder<'a> {
    #[inline(always)]
    pub fn new(slice: &'a mut [u8]) -> Self {
        Self { slice: slice, position: 0 }
    }
}

impl<'a> BufEncoder<'a> {
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.slice.len()
    }

    #[inline(always)]
    pub fn remaining(&self) -> usize {
        self.slice.len() - self.position
    }

    pub fn get_content(&self) -> &[u8] {
        &self.slice[0 .. self.position]
    }

    #[inline(always)]
    pub unsafe fn reserve_space<T: Sized + Copy>(&mut self) -> Result<(*mut T), (usize, usize)>
        where T: Sized + Copy
    {
        let size = size_of::<T>();
        let remaining = self.remaining();
        if remaining < size {
            return Err((remaining, size));
        }

        let position = self.position;
        self.position += size;

        Ok((&mut self.slice[position] as *mut u8) as *mut T)
    }

    #[inline(always)]
    pub unsafe fn reserve_space_by_size(&mut self, size: usize) -> Result<*mut u8, (usize, usize)>
    {
        let remaining = self.remaining();
        if remaining < size {
            return Err((remaining, size));
        }

        let position = self.position;
        self.position += size;

        Ok(&mut self.slice[position] as *mut u8)
    }

    #[inline(always)]
    pub fn put<T>(&mut self, item: &T) -> Result<(), (usize, usize)>
        where T: Sized + Copy
    {
        let size = size_of::<T>();
        let remaining = self.remaining();
        if remaining < size {
            return Err((remaining, size));
        }

        let dest = &mut self.slice[self.position];

        unsafe {
            ::std::ptr::write_unaligned((dest as *mut u8) as *mut T, item.clone());
        }

        self.position += size;
        Ok(())
    }

    pub fn append_bytes(&mut self, slice: &[u8]) -> Result<(), (usize, usize)> {
        let size = slice.len();
        let remaining = self.remaining();
        if remaining < size {
            return Err((remaining, size));
        }

        let dest = &mut self.slice[self.position];

        unsafe {
            ::std::ptr::copy(&slice[0], dest as *mut u8, size);
        }

        self.position += size;
        Ok(())

    }
}

//////////////////////////////////////////////////////////////////////////
// BufDecoder
//

#[derive(Clone)]
pub struct BufDecoder<'a> {
    slice: &'a [u8],
    position: usize,
}

impl<'a> BufDecoder<'a> {
    pub fn new(slice: &'a [u8]) -> Self {
        Self { slice: slice, position: 0 }
    }
}

impl<'a> BufDecoder<'a> {
    pub fn len(&self) -> usize {
        self.slice.len()
    }

    pub fn remaining(&self) -> usize {
        self.slice.len() - self.position
    }

    pub fn get_slice(&mut self, size: usize) -> Result<&[u8], (usize, usize)>
    {
        let remaining = self.remaining();
        if remaining < size {
            return Err((remaining, size));
        }

        let value = &self.slice[self.position .. self.position + size];
        self.position += size;
        Ok(value)
    }

    pub fn get_remaining_slice(&mut self) -> Result<&[u8], (usize, usize)>
    {
        let remaining = self.remaining();
        self.get_slice(remaining)
    }

    pub fn get<T>(&mut self) -> Result<T, (usize, usize)>
        where T: Sized + Copy
    {
        let size = size_of::<T>();
        let remaining = self.remaining();
        if remaining < size {
            return Err((remaining, size));
        }

        let source = &self.slice[self.position];

        let value = unsafe {
            ::std::ptr::read((source as *const u8) as *const T)
        };

        self.position += size;

        Ok(value)
    }
}


