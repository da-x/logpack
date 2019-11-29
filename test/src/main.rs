use logpack_derive::Logpack;
use serde_derive::Deserialize;

use hexdump::HexReader;
use ansi_term::{ANSIString, ANSIStrings};
use ron::ser::{to_string};
use ron::de::{from_str};
use std::fmt::Debug;
use std::io::BufRead;

#[derive(Logpack, Debug, Eq, PartialEq, Deserialize)]
pub struct GenericType<T> {
    test: T,
    field: u32,
}

#[derive(Logpack, Debug, Eq, PartialEq, Deserialize)]
pub enum SimpleEnum {
    WithUnit,
    TupleField(u32),
    NamedField {
        some_str: String,
    },
    OtherUnit(SimpleStructUnit),
}

#[derive(Logpack, Debug, Eq, PartialEq, Deserialize)]
pub struct SimpleStructNamed {
    some_str: String,
}

#[derive(Logpack, Debug, Eq, PartialEq, Deserialize)]
pub struct SimpleStructTuple(u32, String);

#[derive(Logpack, Debug, Eq, PartialEq, Deserialize)]
pub struct SimpleStructUnit;

fn test<E>(st: &mut logpack::SeenTypes,
           tm: &mut logpack::NameMap,
           e: &E)
    where E: logpack::Logpack + logpack::Encoder +
             serde::de::DeserializeOwned + Eq + PartialEq + Debug
{
    use logpack::*;

    let type_desc = Logpack::logpack_describe_by_value(e, st);
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

fn test_ser_only<E>(st: &mut logpack::SeenTypes,
                    tm: &mut logpack::NameMap,
                    e: &E)
    where E: logpack::Logpack + logpack::Encoder + Debug
{
    use logpack::*;

    let type_desc = Logpack::logpack_describe_by_value(e, st);
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
    println!("Size in bytes of Packlog binary: {:?}", sizer_result);

    assert_eq!(encoded.len(), sizer_result);
}

#[derive(Logpack, Debug)]
pub struct StaticRecord {
    pub file: &'static str,
    pub line: u32,
    pub function: &'static str,
    pub module: &'static str,
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

    let sr = StaticRecord {
        file : "file.rs",
        line : 123,
        function : "func",
        module: "mod",
    };

    test_ser_only(&mut st, &mut tm, &sr);
}
