#[derive(Eq, PartialEq, Ord, PartialOrd, Hash, Clone)]
pub struct Key(Vec<u8>);

/// Value type which is essentially raw bytes.
pub type Value = Vec<u8>;

/// Key-value pair type.
///
/// The value is simply raw bytes;
pub type KvPair = (Vec<u8>, Value);

impl Key {
    #[inline]
    pub fn from_raw(key: &[u8]) -> Key {
        Key(key.to_vec())
    }

    #[inline]
    pub fn as_raw(&self) -> &Vec<u8> {
        &self.0
    }

    #[inline]
    pub fn into_raw(self) -> Vec<u8> {
        self.0
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn key_works() {
        let result = b"";
        assert_eq!(Key::from_raw(result).into_raw(), b"");
        assert_eq!(Key::from_raw(b"a").into_raw(), b"a");
    }
}
