use std::collections::HashMap;
use std::fmt;
use std::io::Read;
use std::io;
use std::ops::Index;
use std::string::{ self, ToString };

use byteorder::{ BigEndian, ReadBytesExt };
use flate2::read::{ GzDecoder, ZlibDecoder };
use rustc_serialize;
use rustc_serialize::hex::ToHex;

use self::DecoderError::*;

/// Represents a NBT value
#[derive(Clone, PartialEq)]
pub enum Nbt {
    Byte(i8),
    Short(i16),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    ByteArray(Vec<u8>),
    IntArray(Vec<i32>),
    String(String),
    List(List),
    Compound(Compound)
}

impl fmt::Debug for Nbt {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Nbt::Byte(x) => write!(f, "{}b", x),
            Nbt::Short(x) => write!(f, "{}s", x),
            Nbt::Int(x) => write!(f, "{}i", x),
            Nbt::Long(x) => write!(f, "{}L", x),
            Nbt::Float(x) => write!(f, "{:.1}f", x),
            Nbt::Double(x) => write!(f, "{:.1}", x),
            Nbt::ByteArray(ref x) => write!(f, "b<{}>", x[..].to_hex()),
            Nbt::IntArray(ref x) => write!(f, "{:?}", *x),
            Nbt::String(ref x) => write!(f, "\"{}\"", *x),
            Nbt::List(ref x) => write!(f, "{:?}", *x),
            Nbt::Compound(ref x) => write!(f, "{:?}", *x)
        }
    }
}

impl fmt::Display for Nbt {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Nbt::Byte(x) => write!(f, "{}", x),
            Nbt::Short(x) => write!(f, "{}", x),
            Nbt::Int(x) => write!(f, "{}", x),
            Nbt::Long(x) => write!(f, "{}", x),
            Nbt::Float(x) => write!(f, "{:.1}", x),
            Nbt::Double(x) => write!(f, "{:.1}", x),
            Nbt::ByteArray(ref x) => write!(f, "<{}>", x[..].to_hex()),
            Nbt::IntArray(ref x) => write!(f, "{:?}", *x),
            Nbt::String(ref x) => write!(f, "\"{}\"", *x),
            Nbt::List(ref x) => write!(f, "{:?}", *x),
            Nbt::Compound(ref x) => write!(f, "{:?}", *x)
        }
    }
}

/// An ordered list of NBT values.
#[derive(Clone, PartialEq, Debug)]
pub enum List {
    Byte(Vec<i8>),
    Short(Vec<i16>),
    Int(Vec<i32>),
    Long(Vec<i64>),
    Float(Vec<f32>),
    Double(Vec<f64>),
    ByteArray(Vec<Vec<u8>>),
    IntArray(Vec<Vec<i32>>),
    String(Vec<String>),
    List(Vec<List>),
    Compound(Vec<Compound>)
}

/// An unordered list of named NBT values.
pub type Compound = HashMap<String, Nbt>;

impl Nbt {
    pub fn from_reader<R: Read>(r: R) -> NbtReaderResult<Nbt> {
        Ok(try!(NbtReader::new(r).tag()).unwrap().0)
    }

    pub fn from_gzip(data: &[u8]) -> NbtReaderResult<Nbt> {
        let reader = GzDecoder::new(data).unwrap();
        Nbt::from_reader(reader)
    }

    pub fn from_zlib(data: &[u8]) -> NbtReaderResult<Nbt> {
        let reader = ZlibDecoder::new(data);
        Nbt::from_reader(reader)
    }

    pub fn as_byte(&self) -> Option<i8> {
        match *self { Nbt::Byte(b) => Some(b), _ => None }
    }

    pub fn into_compound(self) -> Result<Compound, Nbt> {
        match self { Nbt::Compound(c) => Ok(c), x => Err(x) }
    }

    pub fn into_compound_list(self) -> Result<Vec<Compound>, Nbt> {
        match self { Nbt::List(List::Compound(c)) => Ok(c), x => Err(x) }
    }

    pub fn as_bytearray(&self) -> Option<&[u8]> {
        match *self { Nbt::ByteArray(ref b) => Some(&b[..]), _ => None }
    }

    pub fn into_bytearray(self) -> Result<Vec<u8>, Nbt> {
        match self { Nbt::ByteArray(b) => Ok(b), x => Err(x) }
    }

    pub fn as_float_list(&self) -> Option<&[f32]> {
        match *self { Nbt::List(List::Float(ref f)) => Some(&f[..]), _ => None }
    }

    pub fn as_double_list(&self) -> Option<&[f64]> {
        match *self { Nbt::List(List::Double(ref d)) => Some(&d[..]), _ => None }
    }
}

impl<'a> Index<&'a str> for Nbt {
    type Output = Nbt;

    fn index<'b>(&'b self, s: &'a str) -> &'b Nbt {
        match *self {
            Nbt::Compound(ref c) => c.get(s).unwrap(),
            _ => panic!("cannot index non-compound Nbt ({:?}) with '{}'", self, s)
        }
    }
}

const TAG_END: i8 = 0;
const TAG_BYTE: i8 = 1;
const TAG_SHORT: i8 = 2;
const TAG_INT: i8 = 3;
const TAG_LONG: i8 = 4;
const TAG_FLOAT: i8 = 5;
const TAG_DOUBLE: i8 = 6;
const TAG_BYTE_ARRAY: i8 = 7;
const TAG_STRING: i8 = 8;
const TAG_LIST: i8 = 9;
const TAG_COMPOUND: i8 = 10;
const TAG_INT_ARRAY: i8 = 11;

pub type NbtReaderResult<T> = Result<T, NbtReaderError>;

#[derive(Debug)]
pub enum NbtReaderError {
    Io(io::Error),
    Utf8(string::FromUtf8Error),
}

impl From<io::Error> for NbtReaderError {
    fn from(err: io::Error) -> NbtReaderError { NbtReaderError::Io(err) }
}

impl From<string::FromUtf8Error> for NbtReaderError {
    fn from(err: string::FromUtf8Error) -> NbtReaderError { NbtReaderError::Utf8(err) }
}

pub struct NbtReader<R> {
    reader: R
}

impl<R: Read> NbtReader<R> {
    pub fn new(reader: R) -> NbtReader<R> {
        NbtReader { reader }
    }

    fn i8(&mut self) -> NbtReaderResult<i8> { self.reader.read_i8().map_err(NbtReaderError::from) }
    fn i16(&mut self) -> NbtReaderResult<i16> { self.reader.read_i16::<BigEndian>().map_err(NbtReaderError::from) }
    fn i32(&mut self) -> NbtReaderResult<i32> { self.reader.read_i32::<BigEndian>().map_err(NbtReaderError::from) }
    fn i64(&mut self) -> NbtReaderResult<i64> { self.reader.read_i64::<BigEndian>().map_err(NbtReaderError::from) }
    fn f32(&mut self) -> NbtReaderResult<f32> { self.reader.read_f32::<BigEndian>().map_err(NbtReaderError::from) }
    fn f64(&mut self) -> NbtReaderResult<f64> { self.reader.read_f64::<BigEndian>().map_err(NbtReaderError::from) }

    fn string(&mut self) -> NbtReaderResult<String> {
        let len = try!(self.reader.read_u16::<BigEndian>()) as usize;
        let mut v = Vec::with_capacity(len);
        for _ in 0..len {
            let mut c = [0];
            try!(self.reader.read(&mut c));
            v.push(c[0])
        }
        String::from_utf8(v).map_err(NbtReaderError::from)
    }

    fn array_u8(&mut self) -> NbtReaderResult<Vec<u8>> {
        let len = try!(self.i32()) as usize;
        let mut v = Vec::with_capacity(len);
        for _ in 0..len {
            let mut c = [0];
            try!(self.reader.read(&mut c));
            v.push(c[0])
        }
        Ok(v)
    }

    fn array<T, F>(&mut self, mut read: F) -> NbtReaderResult<Vec<T>>
        where F: FnMut(&mut NbtReader<R>) -> NbtReaderResult<T>
    {
        let len = try!(self.i32()) as usize;
        let mut v = Vec::with_capacity(len);
        for _ in 0..len {
            v.push(try!(read(self)))
        }
        Ok(v)
    }

    fn compound(&mut self) -> NbtReaderResult<Compound> {
        let mut map = HashMap::new();
        while let Some((v, name)) = try!(self.tag()) {
            map.insert(name, v);
        }
        Ok(map)
    }

    fn list(&mut self) -> NbtReaderResult<List> {
        match try!(self.i8()) {
            TAG_END => {
                assert_eq!(try!(self.i32()), 0);
                Ok(List::Compound(Vec::new()))
            }
            TAG_BYTE => self.array(|r| r.i8()).map(List::Byte),
            TAG_SHORT => self.array(|r| r.i16()).map(List::Short),
            TAG_INT => self.array(|r| r.i32()).map(List::Int),
            TAG_LONG => self.array(|r| r.i64()).map(List::Long),
            TAG_FLOAT => self.array(|r| r.f32()).map(List::Float),
            TAG_DOUBLE => self.array(|r| r.f64()).map(List::Double),
            TAG_BYTE_ARRAY => self.array(|r| r.array_u8()).map(List::ByteArray),
            TAG_INT_ARRAY => self.array(|r| r.array(|r| r.i32())).map(List::IntArray),
            TAG_STRING => self.array(|r| r.string()).map(List::String),
            TAG_LIST => self.array(|r| r.list()).map(List::List),
            TAG_COMPOUND => self.array(|r| r.compound()).map(List::Compound),
            tag_type => panic!("Unexpected tag type {}", tag_type)
        }
    }

    pub fn tag(&mut self) -> NbtReaderResult<Option<(Nbt, String)>> {
        Ok(match try!(self.i8()) {
            TAG_END => None,
            tag_type => {
                let name = try!(self.string());
                Some((try!(match tag_type {
                    TAG_BYTE => self.i8().map(Nbt::Byte),
                    TAG_SHORT => self.i16().map(Nbt::Short),
                    TAG_INT => self.i32().map(Nbt::Int),
                    TAG_LONG => self.i64().map(Nbt::Long),
                    TAG_FLOAT => self.f32().map(Nbt::Float),
                    TAG_DOUBLE => self.f64().map(Nbt::Double),
                    TAG_BYTE_ARRAY => self.array_u8().map(Nbt::ByteArray),
                    TAG_INT_ARRAY => self.array(|r| r.i32()).map(Nbt::IntArray),
                    TAG_STRING => self.string().map(Nbt::String),
                    TAG_LIST => self.list().map(Nbt::List),
                    TAG_COMPOUND => self.compound().map(Nbt::Compound),
                    tag_type => panic!("Unexpected tag type {}", tag_type)
                }), name))
            }
        })
    }
}

/// A structure to decode NBT to values in rust.
pub struct Decoder {
    stack: Vec<DecodeResult<Nbt>>
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum DecoderError {
    ExpectedError(String, String),
    MissingFieldError(String),
    UnknownVariantError(String),
    ApplicationError(String)
}

pub type DecodeResult<T> = Result<T, DecoderError>;

impl Decoder {
    /// Creates a new decoder instance for decoding the specified NBT value.
    pub fn new(nbt: Nbt) -> Decoder {
        Decoder {
            stack: vec![Ok(nbt)]
        }
    }
    fn pop(&mut self) -> DecodeResult<Nbt> {
        self.stack.pop().unwrap()
    }
    fn push(&mut self, nbt: Nbt) {
        self.stack.push(Ok(nbt))
    }
    fn push_all<T, F>(&mut self, list: Vec<T>, f: F) -> usize
        where F: FnMut(T) -> Nbt
    {
        let len = list.len();
        self.stack.extend(list.into_iter().rev().map(f).map(Ok::<Nbt, DecoderError>));
        len
    }
}

// impl Decodable for Nbt {
//     fn decode<D: serialize::Decoder>(d: &mut D) -> Result<Self, serialize::Decoder::Error> {
//         d.pop()
//     }
// }

macro_rules! expect(
    ($s:expr, $t:path) => ({
        match $s.pop() {
            Ok($t(v)) => Ok(v),
            Ok(other) => {
                Err(ExpectedError(stringify!($t).to_string(), other.to_string()))
            }
            Err(e) => Err(e)
        }
    });
    ($s:expr, $t:ident as $to:ty) => (expect!($s, $t).map(|x| x as $to))
);

impl rustc_serialize::Decoder for Decoder {
    type Error = DecoderError;

    fn read_nil(&mut self) -> DecodeResult<()> {
        Err(ExpectedError("()".to_string(), try!(self.pop()).to_string()))
    }

    fn read_u64(&mut self) -> DecodeResult<u64> { expect!(self, Nbt::Long).map(|x| x as u64) }
    fn read_u32(&mut self) -> DecodeResult<u32> { expect!(self, Nbt::Int).map(|x| x as u32) }
    fn read_u16(&mut self) -> DecodeResult<u16> { expect!(self, Nbt::Short).map(|x| x as u16) }
    fn read_u8 (&mut self) -> DecodeResult<u8>  { expect!(self, Nbt::Byte).map(|x| x as u8) }

    fn read_i64(&mut self) -> DecodeResult<i64> { expect!(self, Nbt::Long) }
    fn read_i32(&mut self) -> DecodeResult<i32> { expect!(self, Nbt::Int) }
    fn read_i16(&mut self) -> DecodeResult<i16> { expect!(self, Nbt::Short) }
    fn read_i8 (&mut self) -> DecodeResult<i8>  { expect!(self, Nbt::Byte) }

    fn read_isize(&mut self) -> DecodeResult<isize> {
        match try!(self.pop()) {
            Nbt::Byte(x) => Ok(x as isize),
            Nbt::Short(x) => Ok(x as isize),
            Nbt::Int(x) => Ok(x as isize),
            Nbt::Long(x) => Ok(x as isize),
            other => Err(ExpectedError("isize".to_string(), other.to_string()))
        }
    }
    fn read_usize(&mut self) -> DecodeResult<usize> {
        Ok(try!(self.read_isize()) as usize)
    }

    fn read_bool(&mut self) -> DecodeResult<bool> {
        Ok(try!(self.read_u8()) != 0)
    }

    fn read_f64(&mut self) -> DecodeResult<f64> { expect!(self, Nbt::Double) }
    fn read_f32(&mut self) -> DecodeResult<f32> { expect!(self, Nbt::Float) }

    fn read_char(&mut self) -> DecodeResult<char> {
        let s = try!(self.read_str());
        {
            let mut it = s.chars();
            match (it.next(), it.next()) {
                // exactly one character
                (Some(c), None) => return Ok(c),
                _ => ()
            }
        }
        Err(ExpectedError("single character string".to_string(), s))
    }

    fn read_str(&mut self) -> DecodeResult<String> {
        expect!(self, Nbt::String)
    }

    fn read_enum<T, F>(&mut self, _name: &str, f: F) -> DecodeResult<T>
        where F: FnOnce(&mut Self) -> DecodeResult<T>
    {
        f(self)
    }

    fn read_enum_variant<T, F>(&mut self, names: &[&str], mut f: F) -> DecodeResult<T>
        where F: FnMut(&mut Self, usize) -> DecodeResult<T>
    {
        let name = match try!(self.pop()) {
            Nbt::String(s) => s,
            Nbt::Compound(mut o) => {
                let name = match o.remove("variant") {
                    Some(Nbt::String(s)) => s,
                    Some(val) => return Err(ExpectedError("String".to_string(), val.to_string())),
                    None => return Err(MissingFieldError("variant".to_string()))
                };
                match o.remove("fields") {
                    Some(v) => {
                        self.push(v);
                        try!(self.read_seq(|_, _| Ok(())));
                    },
                    None => return Err(MissingFieldError("fields".to_string()))
                }
                name
            }
            nbt => {
                return Err(ExpectedError("String or Compound".to_string(), nbt.to_string()))
            }
        };
        let idx = match names.iter().position(|n| n == &name) {
            Some(idx) => idx,
            None => return Err(UnknownVariantError(name))
        };
        f(self, idx)
    }

    fn read_enum_variant_arg<T, F>(&mut self, _idx: usize, f: F) -> DecodeResult<T>
        where F: FnOnce(&mut Self) -> DecodeResult<T>
    {
        f(self)
    }

    fn read_enum_struct_variant<T, F>(&mut self, names: &[&str], f: F) -> DecodeResult<T>
        where F: FnMut(&mut Self, usize) -> DecodeResult<T>
    {
        self.read_enum_variant(names, f)
    }


    fn read_enum_struct_variant_field<T, F>(&mut self, _name: &str, idx: usize, f: F) -> DecodeResult<T>
        where F: FnOnce(&mut Self) -> DecodeResult<T>
    {
        self.read_enum_variant_arg(idx, f)
    }

    fn read_struct<T, F>(&mut self, _name: &str, _len: usize, f: F) -> DecodeResult<T>
        where F: FnOnce(&mut Self) -> DecodeResult<T>
    {
        let value = try!(f(self));
        let _ = self.pop();
        Ok(value)
    }

    fn read_struct_field<T, F>(&mut self, name: &str, _idx: usize, f: F) -> DecodeResult<T>
        where F: FnOnce(&mut Self) -> DecodeResult<T>
    {
        let mut obj = try!(expect!(self, Nbt::Compound));

        let value = match obj.remove(name) {
            None => return Err(MissingFieldError(name.to_string())),
            Some(v) => {
                self.stack.push(Ok(v));
                try!(f(self))
            }
        };
        self.push(Nbt::Compound(obj));
        Ok(value)
    }

    fn read_tuple<T, F>(&mut self, tuple_len: usize, f: F) -> DecodeResult<T>
        where F: FnOnce(&mut Self) -> DecodeResult<T>
    {
        self.read_seq(move |d, len| {
            if len == tuple_len {
                f(d)
            } else {
                Err(ExpectedError(format!("Tuple{}", tuple_len), format!("Tuple{}", len)))
            }
        })
    }

    fn read_tuple_arg<T, F>(&mut self, idx: usize, f: F) -> DecodeResult<T>
        where F: FnOnce(&mut Self) -> DecodeResult<T>
    {
        self.read_seq_elt(idx, f)
    }

    fn read_tuple_struct<T, F>(&mut self, _name: &str, len: usize, f: F) -> DecodeResult<T>
        where F: FnOnce(&mut Self) -> DecodeResult<T>
    {
        self.read_tuple(len, f)
    }

    fn read_tuple_struct_arg<T, F>(&mut self, idx: usize, f: F) -> DecodeResult<T>
        where F: FnOnce(&mut Self) -> DecodeResult<T>
    {
        self.read_tuple_arg(idx, f)
    }

    fn read_option<T, F>(&mut self, mut f: F) -> DecodeResult<T>
        where F: FnMut(&mut Self, bool) -> DecodeResult<T>
    {
        match self.pop() {
            Ok(value) => { self.push(value); f(self, true) }
            Err(MissingFieldError(_)) => f(self, false),
            Err(e) => Err(e)
        }
    }

    fn read_seq<T, F>(&mut self, f: F) -> DecodeResult<T>
        where F: FnOnce(&mut Self, usize) -> DecodeResult<T>
    {
        let len = match try!(expect!(self, Nbt::List)) {
            List::Byte(list) => self.push_all(list, Nbt::Byte),
            List::Short(list) => self.push_all(list, Nbt::Short),
            List::Int(list) => self.push_all(list, Nbt::Int),
            List::Long(list) => self.push_all(list, Nbt::Long),
            List::Float(list) => self.push_all(list, Nbt::Float),
            List::Double(list) => self.push_all(list, Nbt::Double),
            List::ByteArray(list) => self.push_all(list, Nbt::ByteArray),
            List::IntArray(list) => self.push_all(list, Nbt::IntArray),
            List::String(list) => self.push_all(list, Nbt::String),
            List::List(list) => self.push_all(list, Nbt::List),
            List::Compound(list) => self.push_all(list, Nbt::Compound)
        };
        f(self, len)
    }

    fn read_seq_elt<T, F>(&mut self, _idx: usize, f: F) -> DecodeResult<T>
        where F: FnOnce(&mut Self) -> DecodeResult<T>
    {
        f(self)
    }

    fn read_map<T, F>(&mut self, f: F) -> DecodeResult<T>
        where F: FnOnce(&mut Self, usize) -> DecodeResult<T>
    {
        let obj = try!(expect!(self, Nbt::Compound));
        let len = obj.len();
        for (key, value) in obj.into_iter() {
            self.push(value);
            self.push(Nbt::String(key));
        }
        f(self, len)
    }

    fn read_map_elt_key<T, F>(&mut self, _idx: usize, f: F) -> DecodeResult<T>
        where F: FnOnce(&mut Self) -> DecodeResult<T>
    {
        f(self)
    }

    fn read_map_elt_val<T, F>(&mut self, _idx: usize, f: F) -> DecodeResult<T>
        where F: FnOnce(&mut Self) -> DecodeResult<T>
    {
        f(self)
    }

    fn error(&mut self, err: &str) -> DecoderError {
        ApplicationError(err.to_string())
    }
}
