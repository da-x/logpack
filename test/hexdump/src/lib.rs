// Based on stuff from: https://www.snip2code.com/Snippet/1473242/Rust-Hexdump

use std::io::{self, Read, BufRead};
use std::cmp;
use std::fmt::{self, Write};

const HR_BYTES_PER_LINE: usize = 16;

pub struct HexReader<T> {
    inner: T,
    buf: String,
    buf_pos: usize,
    line_count: usize,
}

impl<T: Read> HexReader<T> {
    pub fn new(inner: T) -> HexReader<T> {
        HexReader {
            inner: inner,
            buf: String::new(),
            buf_pos: 0,
            line_count: 0
        }
    }

    pub fn render_bytes(&mut self, bytes: &[u8]) -> fmt::Result {
        write!(&mut self.buf, "${:08x} ", HR_BYTES_PER_LINE * self.line_count)?;
        for (count, b) in bytes.iter().enumerate() {
            if count == 8 {
                write!(&mut self.buf, " ")?;
            }
            write!(&mut self.buf, " {:02x}", b)?;
        }
        loop {
            if self.buf.len() > 60 { break }
            write!(&mut self.buf, " ")?;
        }
        write!(&mut self.buf, "|")?;
        for b in bytes.iter() {
            if *b >= 32 && *b <= 127 {
                write!(&mut self.buf, "{}", *b as char)?;
                continue;
            }
            write!(&mut self.buf, ".")?;
        }
        write!(&mut self.buf, "|")?;
        write!(&mut self.buf, "\n")?;
        self.line_count += 1;
        Ok(())
    }
}

impl<T: Read> Read for HexReader<T> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let nread = {
            let mut rem = self.fill_buf()?;
            rem.read(buf)?
        };
        self.consume(nread);
        Ok(nread)
    }
}

impl<R: Read> BufRead for HexReader<R> {
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        if self.buf_pos >= self.buf.len() {
            let mut nread: usize = 0;
            let mut tmp: [u8; HR_BYTES_PER_LINE] = [0; HR_BYTES_PER_LINE];
            loop {
                nread += match self.inner.read(&mut tmp[nread..]) {
                    Ok(0) if nread == 0 => return Ok(&[]),
                    Ok(0) => break,
                    Ok(n) => n,
                    Err(e) => return Err(e),
                };
                if nread >= HR_BYTES_PER_LINE { break }
            }
            self.buf.clear();
            self.render_bytes(&tmp[..nread]).expect("TODO:");
            self.buf_pos = 0;
        }
        Ok(self.buf[self.buf_pos..].as_bytes())
    }

    fn consume(&mut self, count: usize) {
        self.buf_pos = cmp::min(self.buf_pos + count, self.buf.len());
    }
}
