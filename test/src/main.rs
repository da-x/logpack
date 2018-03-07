#[macro_use]
extern crate logpack_derive;
#[macro_use]
extern crate serde_derive;

extern crate logpack;
extern crate ron;
extern crate logpack_ron;
extern crate serde;
extern crate ansi_term;

use ansi_term::{ANSIString, ANSIStrings};
use self::ron::ser::{to_string};
use self::ron::de::{from_str};
use std::fmt::Debug;

#[derive(LogpackType, Debug, Eq, PartialEq, Deserialize)]
pub enum SimpleEnum {
    WithUnit,
    TupleField(u32),
    NamedField {
        some_str: String,
    },
    OtherUnit(SimpleStructUnit),
}

#[derive(LogpackType, Debug, Eq, PartialEq, Deserialize)]
pub struct SimpleStructNamed {
    some_str: String,
}

#[derive(LogpackType, Debug, Eq, PartialEq, Deserialize)]
pub struct SimpleStructTuple(u32, String);

#[derive(LogpackType, Debug, Eq, PartialEq, Deserialize)]
pub struct SimpleStructUnit;

fn test<E>(st: &mut logpack::SeenTypes,
           tm: &mut logpack::NameMap,
           e: &E)
    where E: logpack::LogpackType + logpack::Encoder +
             serde::de::DeserializeOwned + Eq + PartialEq + Debug
{
    use logpack::*;

    let type_desc = LogpackType::logpack_describe_by_value(&e, st);
    let type_ser = to_string(&type_desc).expect("Serialization failed");
    let mut bytes : [u8; 1024] = [0; 1024];
    let mut enc_buf = logpack::BufEncoder::new(&mut bytes);

    e.logpack_encode(&mut enc_buf).unwrap();
    let sizer_result = e.logpack_sizer();

    let encoded = enc_buf.get_content();
    let deser_type = from_str(type_ser.as_str()).unwrap();
    let deser_type = tm.feed(deser_type).unwrap();

    let repr_output = {
        let mut tmp = String::new();
        {
            let dec_buf = logpack::BufDecoder::new(encoded);
            let mut decoder = logpack::Decoder::new(tm, dec_buf);
            let mut repr = logpack_ron::Repr::new(&mut tmp);
            decoder.decode(&deser_type, &mut repr).unwrap();
        }
        tmp
    };

    let repr_ansi_output = {
        let mut tmp : Vec<ANSIString<'static>> = Vec::new();
        {
            let dec_buf = logpack::BufDecoder::new(encoded);
            let mut decoder = logpack::Decoder::new(tm, dec_buf);
            let mut repr = logpack_ron::ansi::Repr::new(&mut tmp).with_enum_names();
            decoder.decode(&deser_type, &mut repr).unwrap();
        }
        tmp
    };

    println!("");
    println!("Value to encode (Debug repr): {:?}", *e);
    println!("Serialized Type in 'ron' (None = type already seen): {}", type_ser);
    let r = HexReader::new(encoded);
    for line in r.lines() {
        println!("Binary value in hex: {}", line.unwrap());
    }
    println!("Packlog Deser Output: {}", repr_output);
    println!("Packlog Deser ANSI output: {}", ANSIStrings(repr_ansi_output.as_slice()));
    let deser : E = from_str(repr_output.as_str()).unwrap();
    println!("Debug repr after 'ron' desering of Packlog deser: {:?}", deser);
    println!("Size in bytes of Packlog binary: {:?}", sizer_result);

    assert_eq!(deser, *e);
    assert_eq!(encoded.len(), sizer_result);
}

fn main()
{
    let mut st = logpack::SeenTypes::new();
    let mut tm = logpack::NameMap::new();

    test(&mut st, &mut tm, &SimpleEnum::WithUnit);
    test(&mut st, &mut tm, &SimpleEnum::TupleField(30));
    test(&mut st, &mut tm, &SimpleEnum::NamedField { some_str: String::from("test") });
    test(&mut st, &mut tm, &SimpleStructNamed { some_str: String::from("bla") });
    test(&mut st, &mut tm, &Some(SimpleStructNamed { some_str: String::from("bla") }));
    test(&mut st, &mut tm, &SimpleStructTuple(123, String::from("bla")));
    test(&mut st, &mut tm, &SimpleStructUnit);
    test(&mut st, &mut tm, &SimpleEnum::OtherUnit(SimpleStructUnit));
    test(&mut st, &mut tm, &Some(10u32));
    test(&mut st, &mut tm, &Some((10u32, (4u8, 12u32))));
}

// From: https://www.snip2code.com/Snippet/1473242/Rust-Hexdump

use std::io::{self, Read, BufRead};
use std::cmp;
use std::fmt::{self, Write};

const HR_BYTES_PER_LINE: usize = 16;

struct HexReader<T> {
    inner: T,
    buf: String,
    buf_pos: usize,
    line_count: usize,
}

impl<T: Read> HexReader<T> {
    fn new(inner: T) -> HexReader<T> {
        HexReader {
            inner: inner,
            buf: String::new(),
            buf_pos: 0,
            line_count: 0
        }
    }

    fn render_bytes(&mut self, bytes: &[u8]) -> fmt::Result {
        try!(write!(&mut self.buf, "${:08x} ", HR_BYTES_PER_LINE * self.line_count));
        for (count, b) in bytes.iter().enumerate() {
            if count == 8 {
                try!(write!(&mut self.buf, " "));
            }
            try!(write!(&mut self.buf, " {:02x}", b));
        }
        loop {
            if self.buf.len() > 60 { break }
            try!(write!(&mut self.buf, " "));
        }
        try!(write!(&mut self.buf, "|................|"));
        try!(write!(&mut self.buf, "\n"));
        self.line_count += 1;
        Ok(())
    }
}

impl<T: Read> Read for HexReader<T> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let nread = {
            let mut rem = try!(self.fill_buf());
            try!(rem.read(buf))
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
