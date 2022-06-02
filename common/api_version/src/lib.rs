// Copyright 2022 Cells
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use cells_types::Key;
use cells_utils::codec::number::{self, NumberEncoder};

#[derive(Default, Clone, Copy)]
pub struct ApiV1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RawValue<T: AsRef<[u8]>> {
    /// The user value.
    pub user_value: T,
    /// The unix timestamp.
    pub ts: Option<u64>,
    /// The status code
    pub status: StatusCode,
    /// The tombstone status
    pub tombstone: bool,
}

impl<T: AsRef<[u8]>> RawValue<T> {
    #[inline]
    pub fn is_valid(&self) -> bool {
        !self.tombstone
    }
}

pub trait KvFormat: Clone + Copy + 'static + Send + Sync {
    fn decode_raw_value(bytes: &[u8]) -> Option<RawValue<&[u8]>>;
    fn encode_raw_value(value: RawValue<&[u8]>) -> Vec<u8>;
    fn encode_raw_value_owned(value: RawValue<Vec<u8>>) -> Vec<u8>;

    fn decode_raw_key(key: &Key) -> Vec<u8> {
        key.as_raw().clone()
    }

    fn encode_raw_key(key: &[u8]) -> Key {
        Key::from_raw(key)
    }
}

impl KvFormat for ApiV1 {
    fn decode_raw_value(bytes: &[u8]) -> Option<RawValue<&[u8]>> {
        let mut rest_len = bytes.len().checked_sub(number::U64_SIZE)?;
        let mut status_slice = &bytes[rest_len..];
        let s = number::decode_u64(&mut status_slice).unwrap_or_default();
        let status = StatusCode::from(s);
        let tombstone = if status.is_tombstone() { true } else { false };

        rest_len = rest_len.checked_sub(number::U64_SIZE)?;
        let mut ts_slice = &bytes[rest_len..rest_len + number::U64_SIZE];
        let ts = number::decode_u64(&mut ts_slice).unwrap_or_default();

        // let status = bytes.len().checked_sub(number::U64_SIZE).and_then(|l| {
        //     rest_len = l;
        //     let mut status_slice = &bytes[l..];
        //     let s = number::decode_u64(&mut status_slice).unwrap_or_default();
        //     Some(StatusCode::from(s))
        // });

        Some(RawValue {
            user_value: &bytes[..rest_len],
            ts: Some(ts),
            status,
            tombstone,
        })
    }

    fn encode_raw_value(value: RawValue<&[u8]>) -> Vec<u8> {
        let mut buf: Vec<u8> =
            Vec::with_capacity(value.user_value.len() + number::U64_SIZE + number::U64_SIZE);

        buf.extend_from_slice(value.user_value);

        let ts = value.ts.unwrap_or_default();
        buf.encode_u64(ts).unwrap();
        let mut status = value.status;
        if value.tombstone {
            status.insert(StatusCode::IS_TOMBSTONE);
        }

        buf.encode_u64(status.bits()).unwrap();

        buf
    }

    fn encode_raw_value_owned(mut value: RawValue<Vec<u8>>) -> Vec<u8> {
        value
            .user_value
            .reserve(number::U64_SIZE + number::U64_SIZE);
        value
            .user_value
            .encode_u64(value.ts.unwrap_or_default())
            .unwrap();
        if value.tombstone {
            value.status.insert(StatusCode::IS_TOMBSTONE);
        }
        value.user_value.encode_u64(value.status.bits()).unwrap();
        value.user_value
    }
}

mod status_code;
use status_code::StatusCode;

#[cfg(test)]
mod tests {

    use crate::{status_code::StatusCode, ApiV1, KvFormat, RawValue};
    use cells_types::Key;

    #[test]
    fn api_v1_works() {
        let k = Key::from_raw(b"123");
        let a = k.as_raw();
        assert_eq!(a, b"123");

        let v = RawValue {
            user_value: &b"123"[..],
            ts: Some(1654045749000),
            status: StatusCode::from_user_status(123),
            tombstone: true,
        };

        let b = ApiV1::encode_raw_value(v);
        // println!("{:?}", b);
        let v1 = ApiV1::decode_raw_value(&b);
        assert!(v1 != None);
        assert_eq!(v1.unwrap().user_value, b"123".to_vec());
        assert_eq!(v1.unwrap().ts, Some(1654045749000));
        assert_eq!(v1.unwrap().status.user_status(), 123u64.into());
        assert_eq!(v1.unwrap().tombstone, true);
        assert!(!v1.unwrap().is_valid());

        // Test Null Value

        let v = RawValue {
            user_value: vec![],
            ts: None,
            status: StatusCode::GOOD,
            tombstone: false,
        };

        let b = ApiV1::encode_raw_value_owned(v);
        assert_eq!(b, &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);

        let v1 = ApiV1::decode_raw_value(&b);
        assert!(v1 != None);
        assert_eq!(v1.unwrap().user_value, b"".to_vec());
        assert_eq!(v1.unwrap().ts, Some(0));
        assert_eq!(v1.unwrap().status, 0u64.into());
        assert!(v1.unwrap().is_valid());
    }
}
