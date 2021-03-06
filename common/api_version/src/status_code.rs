use bitflags::bitflags;
use std::fmt::{self, Formatter};

bitflags! {
    pub struct StatusCode: u64 {

        // Mask for the status code section
        const SYSTEM_STATUS_MASK = 0xff00_0000_0000_0000;
        // Mask for the bits section
        const USER_STATUS_MASK = 0x00ff_ffff_ffff_ffff;

        const IS_TOMBSTONE                = 0x8000_0000_0000_0000;

        const GOOD = 0;
    }
}

impl StatusCode {
    pub fn user_status(&self) -> StatusCode {
        *self & StatusCode::USER_STATUS_MASK
    }

    /// Returns the status only
    pub fn system_status(&self) -> StatusCode {
        *self & StatusCode::SYSTEM_STATUS_MASK
    }

    /// Tests if the status code is tombstone
    pub fn is_tombstone(&self) -> bool {
        self.contains(StatusCode::IS_TOMBSTONE)
    }

    /// Valid user status is 56 bytes, The first 8 bytes are system meta data
    pub fn from_user_status(value: u64) -> Self {
        StatusCode { bits: value } & StatusCode::USER_STATUS_MASK
    }
}

impl Default for StatusCode {
    fn default() -> Self {
        StatusCode::GOOD
    }
}

impl fmt::Display for StatusCode {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let u = self.user_status();
        let s = self.system_status();
        write!(f, "0x{:X} 0x{:X}", s, u)
    }
}

impl From<u64> for StatusCode {
    fn from(value: u64) -> Self {
        StatusCode { bits: value }
    }
}

impl From<StatusCode> for u64 {
    fn from(status: StatusCode) -> Self {
        status.user_status().bits()
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn status_code() {
        let s = StatusCode::default();
        assert_eq!(s, StatusCode::GOOD);

        let s = StatusCode::IS_TOMBSTONE;
        assert!(s.is_tombstone());
    }

    #[test]
    fn status_transform() {
        let s = StatusCode::default();
        assert!(s.is_empty());

        let s = StatusCode::from(64);
        let code: u64 = s.into();
        assert_eq!(code, 64);

        let s: StatusCode = 64u64.into();
        assert_eq!(s.system_status(), StatusCode::GOOD);
        let code: u64 = s.into();
        assert_eq!(code, 64);
    }
}
