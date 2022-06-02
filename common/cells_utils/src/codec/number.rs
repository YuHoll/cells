use byteorder::{BigEndian, ByteOrder, LittleEndian, WriteBytesExt};
use snafu::{ResultExt, Snafu};

use std::{io::Write, mem};

const SIGN_MARK: u64 = 0x8000000000000000;
pub const U64_SIZE: usize = 8;
pub const I64_SIZE: usize = 8;
pub const F64_SIZE: usize = 8;

fn order_encode_i64(v: i64) -> u64 {
    v as u64 ^ SIGN_MARK
}

fn order_decode_i64(u: u64) -> i64 {
    (u ^ SIGN_MARK) as i64
}

fn order_encode_f64(v: f64) -> u64 {
    let u = v.to_bits();
    if v.is_sign_positive() {
        u | SIGN_MARK
    } else {
        !u
    }
}

fn order_decode_f64(u: u64) -> f64 {
    let u = if u & SIGN_MARK > 0 {
        u & (!SIGN_MARK)
    } else {
        !u
    };
    f64::from_bits(u)
}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Encoder Error: {}", source))]
    EncoderNumberFail { source: std::io::Error },

    #[snafu(display("Encoder Error: Unexpected eof"))]
    EncoderUnexpectedEOF,
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

pub trait NumberEncoder: Write {
    fn encode_i64(&mut self, v: i64) -> Result<()> {
        let u = order_encode_i64(v);
        self.encode_u64(u)
    }

    fn encode_u64(&mut self, v: u64) -> Result<()> {
        self.write_u64::<BigEndian>(v)
            .context(EncoderNumberFailSnafu)
    }

    fn encode_f64(&mut self, f: f64) -> Result<()> {
        let u = order_encode_f64(f);
        self.encode_u64(u)
    }

    fn encode_u32(&mut self, v: u32) -> Result<()> {
        self.write_u32::<BigEndian>(v)
            .context(EncoderNumberFailSnafu)
    }

    fn encode_i32(&mut self, v: i32) -> Result<()> {
        self.write_i32::<LittleEndian>(v)
            .context(EncoderNumberFailSnafu)
    }

    fn encode_f32(&mut self, v: f32) -> Result<()> {
        self.write_f32::<LittleEndian>(v)
            .context(EncoderNumberFailSnafu)
    }

    fn encode_u16(&mut self, v: u16) -> Result<()> {
        self.write_u16::<BigEndian>(v)
            .context(EncoderNumberFailSnafu)
    }

    fn encode_i16(&mut self, v: i16) -> Result<()> {
        self.write_i16::<LittleEndian>(v)
            .context(EncoderNumberFailSnafu)
    }
}

impl<T: Write> NumberEncoder for T {}

#[inline]
fn read_num_bytes<T, F>(size: usize, data: &mut &[u8], f: F) -> Result<T>
where
    F: Fn(&[u8]) -> T,
{
    if data.len() >= size {
        let buf = &data[..size];
        *data = &data[size..];
        return Ok(f(buf));
    }
    Err(Error::EncoderUnexpectedEOF)
}

#[inline]
pub fn decode_i64(data: &mut &[u8]) -> Result<i64> {
    decode_u64(data).map(order_decode_i64)
}

#[inline]
pub fn decode_u64(data: &mut &[u8]) -> Result<u64> {
    read_num_bytes(mem::size_of::<u64>(), data, BigEndian::read_u64)
}

#[inline]
pub fn decode_f64(data: &mut &[u8]) -> Result<f64> {
    decode_u64(data).map(order_decode_f64)
}

#[inline]
pub fn decode_u32(data: &mut &[u8]) -> Result<u32> {
    read_num_bytes(mem::size_of::<u32>(), data, BigEndian::read_u32)
}

#[inline]
pub fn decode_i32(data: &mut &[u8]) -> Result<i32> {
    read_num_bytes(mem::size_of::<i32>(), data, LittleEndian::read_i32)
}

#[inline]
pub fn decode_f32(data: &mut &[u8]) -> Result<f32> {
    read_num_bytes(mem::size_of::<f32>(), data, LittleEndian::read_f32)
}

#[inline]
pub fn decode_u16(data: &mut &[u8]) -> Result<u16> {
    read_num_bytes(mem::size_of::<u16>(), data, BigEndian::read_u16)
}

#[inline]
pub fn decode_i16(data: &mut &[u8]) -> Result<i16> {
    read_num_bytes(mem::size_of::<i16>(), data, LittleEndian::read_i16)
}

#[cfg(test)]
mod tests {

    use std::vec;

    use super::*;

    const U64_TESTS: &[u64] = &[
        i64::MIN as u64,
        i64::MAX as u64,
        u64::MIN,
        u64::MAX,
        0,
        1,
        2,
        10,
        20,
        63,
        64,
        65,
        127,
        128,
        129,
        255,
        256,
        257,
        1024,
    ];

    const I64_TESTS: &[i64] = &[
        i64::MIN,
        i64::MAX,
        u64::MIN as i64,
        u64::MAX as i64,
        -1,
        0,
        1,
        2,
        10,
        20,
        63,
        64,
        65,
        127,
        128,
        129,
        255,
        256,
        257,
        1024,
        -1023,
    ];

    const F64_TESTS: &[f64] = &[
        -1.0,
        0.0,
        1.0,
        f64::MAX,
        f64::MIN,
        f32::MAX as f64,
        f32::MIN as f64,
        f64::MIN_POSITIVE,
        f32::MIN_POSITIVE as f64,
        f64::INFINITY,
        f64::NEG_INFINITY,
    ];

    const U32_TESTS: &[u32] = &[
        i32::MIN as u32,
        i32::MAX as u32,
        u32::MIN,
        u32::MAX,
        0,
        1,
        2,
        10,
        20,
        63,
        64,
        65,
        127,
        128,
        129,
        255,
        256,
        257,
        1024,
    ];

    const I32_TESTS: &[i32] = &[
        i32::MIN,
        i32::MAX,
        0,
        1,
        2,
        10,
        20,
        63,
        64,
        65,
        127,
        128,
        129,
        255,
        256,
        257,
        -1024,
    ];

    const F32_TESTS: &[f32] = &[
        -1.0,
        0.0,
        1.0,
        3.1415926,
        f32::MAX,
        f32::MIN,
        f32::MIN_POSITIVE,
        f32::INFINITY,
        f32::NEG_INFINITY,
    ];

    const U16_TESTS: &[u16] = &[
        i16::MIN as u16,
        i16::MAX as u16,
        u16::MIN,
        u16::MAX,
        0,
        1,
        2,
        10,
        20,
        63,
        64,
        65,
        127,
        128,
        129,
        255,
        256,
        257,
        1024,
    ];

    const I16_TESTS: &[i16] = &[
        i16::MIN,
        i16::MAX,
        0,
        1,
        2,
        10,
        20,
        63,
        64,
        65,
        127,
        128,
        129,
        255,
        256,
        257,
        -1024,
    ];

    // #[test]
    // fn u64_encode_decode() {
    //     for &v in U64_TESTS {
    //         let mut buf = vec![];
    //         buf.encode_u64(v).unwrap();
    //         assert!(buf.len() == mem::size_of_val(&v));
    //         assert_eq!(v, decode_u64(&mut buf.as_slice()).unwrap());
    //     }
    // }

    macro_rules! test_serialize {
        ($tag:ident, $enc:ident, $dec:ident, $cases:expr) => {
            #[test]
            fn $tag() {
                for &v in $cases {
                    let mut buf = vec![];
                    buf.$enc(v).unwrap();
                    assert!(buf.len() == mem::size_of_val(&v));
                    assert_eq!(v, $dec(&mut buf.as_slice()).unwrap());
                }
            }
        };
    }

    test_serialize!(u64_serialize, encode_u64, decode_u64, U64_TESTS);
    test_serialize!(i64_serialize, encode_i64, decode_i64, I64_TESTS);
    test_serialize!(f64_serialize, encode_f64, decode_f64, F64_TESTS);
    test_serialize!(u32_serialize, encode_u32, decode_u32, U32_TESTS);
    test_serialize!(i32_serialize, encode_i32, decode_i32, I32_TESTS);
    test_serialize!(f32_serialize, encode_f32, decode_f32, F32_TESTS);
    test_serialize!(u16_serialize, encode_u16, decode_u16, U16_TESTS);
    test_serialize!(i16_serialize, encode_i16, decode_i16, I16_TESTS);
}
