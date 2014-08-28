use flate::{inflate_bytes, inflate_bytes_zlib};
use serialize;
use serialize::Decodable;
use serialize::hex::ToHex;
use std::collections::HashMap;
use std::fmt;
use std::io::{BufReader, IoResult};

/// Represents a NBT value
#[deriving(Clone, PartialEq)]
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

impl fmt::Show for Nbt {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Byte(x) => write!(f, "{}b", x),
            Short(x) => write!(f, "{}s", x),
            Int(x) => write!(f, "{}i", x),
            Long(x) => write!(f, "{}L", x),
            Float(x) => write!(f, "{:.1f}f", x),
            Double(x) => write!(f, "{:.1f}", x),
            ByteArray(ref x) => write!(f, "b<{}>", x.as_slice().to_hex()),
            IntArray(ref x) => write!(f, "{}", *x),
            String(ref x) => write!(f, "\"{}\"", *x),
            List(ref x) => write!(f, "{}", *x),
            Compound(ref x) => write!(f, "{}", *x)
        }
    }
}

#[deriving(Clone, PartialEq, Show)]
pub enum List {
    ByteList(Vec<i8>),
    ShortList(Vec<i16>),
    IntList(Vec<i32>),
    LongList(Vec<i64>),
    FloatList(Vec<f32>),
    DoubleList(Vec<f64>),
    ByteArrayList(Vec<Vec<u8>>),
    IntArrayList(Vec<Vec<i32>>),
    StringList(Vec<String>),
    ListList(Vec<List>),
    CompoundList(Vec<Compound>)
}

pub type Compound = HashMap<String, Nbt>;

impl Nbt {
    pub fn from_reader<R: Reader>(r: &mut R) -> IoResult<Nbt> {
        Ok(try!(NbtReader::new(r).tag()).unwrap().val0())
    }

    pub fn from_gzip(data: &[u8]) -> IoResult<Nbt> {
        assert_eq!(data.slice_to(4), [0x1f, 0x8b, 0x08, 0x00].as_slice());
        let data = inflate_bytes(data.slice_from(10)).expect("inflate failed");
        Nbt::from_reader(&mut BufReader::new(data.as_slice()))
    }

    pub fn from_zlib(data: &[u8]) -> IoResult<Nbt> {
        let data = inflate_bytes_zlib(data).expect("inflate failed");
        Nbt::from_reader(&mut BufReader::new(data.as_slice()))
    }

    pub fn as_byte(&self) -> Option<i8> {
        match *self { Byte(b) => Some(b), _ => None }
    }

    pub fn into_compound(self) -> Result<Compound, Nbt> {
        match self { Compound(c) => Ok(c), x => Err(x) }
    }

    pub fn into_compound_list(self) -> Result<Vec<Compound>, Nbt> {
        match self { List(CompoundList(c)) => Ok(c), x => Err(x) }
    }

    pub fn as_bytearray<'a>(&'a self) -> Option<&'a [u8]> {
        match *self { ByteArray(ref b) => Some(b.as_slice()), _ => None }
    }

    pub fn into_bytearray(self) -> Result<Vec<u8>, Nbt> {
        match self { ByteArray(b) => Ok(b), x => Err(x) }
    }

    pub fn as_float_list<'a>(&'a self) -> Option<&'a [f32]> {
        match *self { List(FloatList(ref f)) => Some(f.as_slice()), _ => None }
    }

    pub fn as_double_list<'a>(&'a self) -> Option<&'a [f64]> {
        match *self { List(DoubleList(ref d)) => Some(d.as_slice()), _ => None }
    }
}

impl<'a> Index<&'a str, Nbt> for Nbt {
    fn index<'b>(&'b self, s: &&'a str) -> &'b Nbt {
        match *self {
            Compound(ref c) => c.find_equiv(s).unwrap(),
            _ => fail!("cannot index non-compound Nbt ({}) with '{}'", self, s)
        }
    }
}

static TAG_END: i8 = 0;
static TAG_BYTE: i8 = 1;
static TAG_SHORT: i8 = 2;
static TAG_INT: i8 = 3;
static TAG_LONG: i8 = 4;
static TAG_FLOAT: i8 = 5;
static TAG_DOUBLE: i8 = 6;
static TAG_BYTE_ARRAY: i8 = 7;
static TAG_STRING: i8 = 8;
static TAG_LIST: i8 = 9;
static TAG_COMPOUND: i8 = 10;
static TAG_INT_ARRAY: i8 = 11;

pub struct NbtReader<'a, R> {
    reader: &'a mut R
}

impl<'a, R: Reader> NbtReader<'a, R> {
    pub fn new(reader: &'a mut R) -> NbtReader<'a, R> {
        NbtReader {
            reader: reader
        }
    }

    fn i8(&mut self) -> IoResult<i8> { self.reader.read_i8() }
    fn i16(&mut self) -> IoResult<i16> { self.reader.read_be_i16() }
    fn i32(&mut self) -> IoResult<i32> { self.reader.read_be_i32() }
    fn i64(&mut self) -> IoResult<i64> { self.reader.read_be_i64() }
    fn f32(&mut self) -> IoResult<f32> { self.reader.read_be_f32() }
    fn f64(&mut self) -> IoResult<f64> { self.reader.read_be_f64() }

    fn string(&mut self) -> IoResult<String> {
        let len = try!(self.reader.read_be_u16()) as uint;
        self.reader.read_exact(len).map(|s| String::from_utf8(s).unwrap())
    }

    fn array_u8(&mut self) -> IoResult<Vec<u8>> {
        let len = try!(self.i32()) as uint;
        self.reader.read_exact(len)
    }

    fn array<T>(&mut self, read: |&mut NbtReader<R>| -> IoResult<T>) -> IoResult<Vec<T>> {
        let len = try!(self.i32()) as uint;
        let mut v = Vec::with_capacity(len);
        for _ in range(0, len) {
            v.push(try!(read(self)))
        }
        Ok(v)
    }

    fn compound(&mut self) -> IoResult<Compound> {
        let mut map = HashMap::new();
        loop {
            match try!(self.tag()) {
                Some((v, name)) => {
                    map.insert(name, v);
                }
                None => break
            }
        }
        Ok(map)
    }

    fn list(&mut self) -> IoResult<List> {
        match try!(self.i8()) {
            TAG_END => {
                assert_eq!(try!(self.i32()), 0);
                Ok(CompoundList(Vec::new()))
            }
            TAG_BYTE => self.array(|r| r.i8()).map(ByteList),
            TAG_SHORT => self.array(|r| r.i16()).map(ShortList),
            TAG_INT => self.array(|r| r.i32()).map(IntList),
            TAG_LONG => self.array(|r| r.i64()).map(LongList),
            TAG_FLOAT => self.array(|r| r.f32()).map(FloatList),
            TAG_DOUBLE => self.array(|r| r.f64()).map(DoubleList),
            TAG_BYTE_ARRAY => self.array(|r| r.array_u8()).map(ByteArrayList),
            TAG_INT_ARRAY => self.array(|r| r.array(|r| r.i32())).map(IntArrayList),
            TAG_STRING => self.array(|r| r.string()).map(StringList),
            TAG_LIST => self.array(|r| r.list()).map(ListList),
            TAG_COMPOUND => self.array(|r| r.compound()).map(CompoundList),
            tag_type => fail!("Unexpected tag type {}", tag_type)
        }
    }

    pub fn tag(&mut self) -> IoResult<Option<(Nbt, String)>> {
        Ok(match try!(self.i8()) {
            TAG_END => None,
            tag_type => {
                let name = try!(self.string());
                Some((try!(match tag_type {
                    TAG_BYTE => self.i8().map(Byte),
                    TAG_SHORT => self.i16().map(Short),
                    TAG_INT => self.i32().map(Int),
                    TAG_LONG => self.i64().map(Long),
                    TAG_FLOAT => self.f32().map(Float),
                    TAG_DOUBLE => self.f64().map(Double),
                    TAG_BYTE_ARRAY => self.array_u8().map(ByteArray),
                    TAG_INT_ARRAY => self.array(|r| r.i32()).map(IntArray),
                    TAG_STRING => self.string().map(String),
                    TAG_LIST => self.list().map(List),
                    TAG_COMPOUND => self.compound().map(Compound),
                    tag_type => fail!("Unexpected tag type {}", tag_type)
                }), name))
            }
        })
    }
}

/// A structure to decode NBT to values in rust.
pub struct Decoder {
    stack: Vec<DecodeResult<Nbt>>
}

#[deriving(Clone, PartialEq, Eq, Show)]
pub enum DecoderError {
    ExpectedError(&'static str, String),
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
}

impl Decoder {
    fn pop(&mut self) -> DecodeResult<Nbt> {
        self.stack.pop().unwrap()
    }
    fn push(&mut self, nbt: Nbt) {
        self.stack.push(Ok(nbt))
    }
    fn push_all<T>(&mut self, list: Vec<T>, f: |T| -> Nbt) -> uint {
        let len = list.len();
        self.stack.extend(list.move_iter().rev().map(f).map(Ok::<Nbt, DecoderError>));
        len
    }
}

impl Decodable<Decoder, DecoderError> for Nbt {
    fn decode(d: &mut Decoder) -> DecodeResult<Nbt> {
        d.pop()
    }
}

macro_rules! expect(
    ($s:expr, $t:ident) => ({
        match $s.pop() {
            Ok($t(v)) => Ok(v),
            Ok(other) => {
                Err(ExpectedError(stringify!($t), other.to_string()))
            }
            Err(e) => Err(e)
        }
    });
    ($s:expr, $t:ident as $to:ty) => (expect!($s, $t).map(|x| x as $to))
)

impl serialize::Decoder<DecoderError> for Decoder {
    fn read_nil(&mut self) -> DecodeResult<()> {
        Err(ExpectedError("()", try!(self.pop()).to_string()))
    }

    fn read_u64(&mut self) -> DecodeResult<u64> { expect!(self, Long as u64) }
    fn read_u32(&mut self) -> DecodeResult<u32> { expect!(self, Int as u32) }
    fn read_u16(&mut self) -> DecodeResult<u16> { expect!(self, Short as u16) }
    fn read_u8 (&mut self) -> DecodeResult<u8 > { expect!(self, Byte as u8) }

    fn read_i64(&mut self) -> DecodeResult<i64> { expect!(self, Long) }
    fn read_i32(&mut self) -> DecodeResult<i32> { expect!(self, Int) }
    fn read_i16(&mut self) -> DecodeResult<i16> { expect!(self, Short) }
    fn read_i8 (&mut self) -> DecodeResult<i8 > { expect!(self, Byte) }

    fn read_int(&mut self) -> DecodeResult<int> {
        match try!(self.pop()) {
            Byte(x) => Ok(x as int),
            Short(x) => Ok(x as int),
            Int(x) => Ok(x as int),
            Long(x) => Ok(x as int),
            other => Err(ExpectedError("int", other.to_string()))
        }
    }
    fn read_uint(&mut self) -> DecodeResult<uint> { Ok(try!(self.read_int()) as uint) }

    fn read_bool(&mut self) -> DecodeResult<bool> {
        Ok(try!(self.read_u8()) != 0)
    }

    fn read_f64(&mut self) -> DecodeResult<f64> { expect!(self, Double) }
    fn read_f32(&mut self) -> DecodeResult<f32> { expect!(self, Float) }

    fn read_char(&mut self) -> DecodeResult<char> {
        let s = try!(self.read_str());
        {
            let mut it = s.as_slice().chars();
            match (it.next(), it.next()) {
                // exactly one character
                (Some(c), None) => return Ok(c),
                _ => ()
            }
        }
        Err(ExpectedError("single character string", s))
    }

    fn read_str(&mut self) -> DecodeResult<String> {
        expect!(self, String)
    }

    fn read_enum<T>(&mut self, _name: &str,
                    f: |&mut Decoder| -> DecodeResult<T>) -> DecodeResult<T> {
        f(self)
    }

    fn read_enum_variant<T>(&mut self,
                            names: &[&str],
                            f: |&mut Decoder, uint| -> DecodeResult<T>)
                            -> DecodeResult<T> {
        let name = match try!(self.pop()) {
            String(s) => s,
            Compound(mut o) => {
                let name = match o.pop_equiv(&"variant") {
                    Some(String(s)) => s,
                    Some(val) => return Err(ExpectedError("String", val.to_string())),
                    None => return Err(MissingFieldError("variant".to_string()))
                };
                match o.pop_equiv(&"fields") {
                    Some(v) => {
                        self.push(v);
                        try!(self.read_seq(|_, _| Ok(())));
                    },
                    None => return Err(MissingFieldError("fields".to_string()))
                }
                name
            }
            nbt => {
                return Err(ExpectedError("String or Compound", nbt.to_string()))
            }
        };
        let idx = match names.iter().position(|n| *n == name.as_slice()) {
            Some(idx) => idx,
            None => return Err(UnknownVariantError(name))
        };
        f(self, idx)
    }

    fn read_enum_variant_arg<T>(&mut self, _idx: uint, f: |&mut Decoder| -> DecodeResult<T>)
                                -> DecodeResult<T> {
        f(self)
    }

    fn read_enum_struct_variant<T>(&mut self, names: &[&str],
                                   f: |&mut Decoder, uint| -> DecodeResult<T>)
                                   -> DecodeResult<T> {
        self.read_enum_variant(names, f)
    }


    fn read_enum_struct_variant_field<T>(&mut self, _name: &str, idx: uint,
                                         f: |&mut Decoder| -> DecodeResult<T>)
                                         -> DecodeResult<T> {
        self.read_enum_variant_arg(idx, f)
    }

    fn read_struct<T>(&mut self, _name: &str, _len: uint,
                      f: |&mut Decoder| -> DecodeResult<T>)
                      -> DecodeResult<T> {
        let value = try!(f(self));
        let _ = self.pop();
        Ok(value)
    }

    fn read_struct_field<T>(&mut self, name: &str, _idx: uint,
                            f: |&mut Decoder| -> DecodeResult<T>)
                            -> DecodeResult<T> {
        let mut obj = try!(expect!(self, Compound));

        let value = match obj.pop_equiv(&name) {
            None => return Err(MissingFieldError(name.to_string())),
            Some(v) => {
                self.stack.push(Ok(v));
                try!(f(self))
            }
        };
        self.push(Compound(obj));
        Ok(value)
    }

    fn read_tuple<T>(&mut self, f: |&mut Decoder, uint| -> DecodeResult<T>) -> DecodeResult<T> {
        self.read_seq(f)
    }

    fn read_tuple_arg<T>(&mut self, idx: uint,
                         f: |&mut Decoder| -> DecodeResult<T>) -> DecodeResult<T> {
        self.read_seq_elt(idx, f)
    }

    fn read_tuple_struct<T>(&mut self, _name: &str,
                            f: |&mut Decoder, uint| -> DecodeResult<T>)
                            -> DecodeResult<T> {
        self.read_tuple(f)
    }

    fn read_tuple_struct_arg<T>(&mut self, idx: uint,
                                f: |&mut Decoder| -> DecodeResult<T>)
                                -> DecodeResult<T> {
        self.read_tuple_arg(idx, f)
    }

    fn read_option<T>(&mut self, f: |&mut Decoder, bool| -> DecodeResult<T>) -> DecodeResult<T> {
        match self.pop() {
            Ok(value) => { self.push(value); f(self, true) }
            Err(MissingFieldError(_)) => f(self, false),
            Err(e) => Err(e)
        }
    }

    fn read_seq<T>(&mut self, f: |&mut Decoder, uint| -> DecodeResult<T>) -> DecodeResult<T> {
        let len = match try!(expect!(self, List)) {
            ByteList(list) => self.push_all(list, Byte),
            ShortList(list) => self.push_all(list, Short),
            IntList(list) => self.push_all(list, Int),
            LongList(list) => self.push_all(list, Long),
            FloatList(list) => self.push_all(list, Float),
            DoubleList(list) => self.push_all(list, Double),
            ByteArrayList(list) => self.push_all(list, ByteArray),
            IntArrayList(list) => self.push_all(list, IntArray),
            StringList(list) => self.push_all(list, String),
            ListList(list) => self.push_all(list, List),
            CompoundList(list) => self.push_all(list, Compound)
        };
        f(self, len)
    }

    fn read_seq_elt<T>(&mut self, _idx: uint,
                       f: |&mut Decoder| -> DecodeResult<T>) -> DecodeResult<T> {
        f(self)
    }

    fn read_map<T>(&mut self, f: |&mut Decoder, uint| -> DecodeResult<T>) -> DecodeResult<T> {
        let obj = try!(expect!(self, Compound));
        let len = obj.len();
        for (key, value) in obj.move_iter() {
            self.push(value);
            self.push(String(key));
        }
        f(self, len)
    }

    fn read_map_elt_key<T>(&mut self, _idx: uint, f: |&mut Decoder| -> DecodeResult<T>)
                           -> DecodeResult<T> {
        f(self)
    }

    fn read_map_elt_val<T>(&mut self, _idx: uint, f: |&mut Decoder| -> DecodeResult<T>)
                           -> DecodeResult<T> {
        f(self)
    }

    fn error(&mut self, err: &str) -> DecoderError {
        ApplicationError(err.to_string())
    }
}
