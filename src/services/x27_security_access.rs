use commkit::TryTo;

use crate::nrc::UdsNrc;
use crate::service::{UdsService, UdsServiceRequest, UdsServiceResponse};
use crate::subfunction::UdsSubfunction;

/*
    ISO 14229-1 Section 9.4

    The SecurityAccess service is used to access data and/or diagnostic services which have restricted access

    Supported NRC:
        - SFNS
        - IMLOIF
        - CNC
        - RSE
        - ROOR
        - IK
        - ENOA
        - RTDNE
*/

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecurityAccessType {
    RequestSeed(u8),
    SendKey(u8),
    Iso26021RequestSeed,
    Iso26021SendKey,
    SystemSupplierSpecific(u8),
}

impl SecurityAccessType {
    pub const fn from_u8(value: u8) -> Option<Self> {
        match value {
            0x01..=0x42 if value % 2 == 1 => Some(Self::RequestSeed(value)),
            0x01..=0x42 => Some(Self::SendKey(value)),
            0x5F => Some(Self::Iso26021RequestSeed),
            0x60 => Some(Self::Iso26021SendKey),
            0x61..=0x7E => Some(Self::SystemSupplierSpecific(value)),
            _ => None,
        }
    }

    pub const fn as_u8(self) -> u8 {
        match self {
            Self::RequestSeed(value) => value,
            Self::SendKey(value) => value,
            Self::Iso26021RequestSeed => 0x5F,
            Self::Iso26021SendKey => 0x60,
            Self::SystemSupplierSpecific(value) => value,
        }
    }

    pub const fn is_request_seed(self) -> bool {
        matches!(self, Self::RequestSeed(_) | Self::Iso26021RequestSeed)
    }

    pub const fn is_send_key(self) -> bool {
        matches!(self, Self::SendKey(_) | Self::Iso26021SendKey)
    }
}

#[allow(non_camel_case_types)]
pub struct x27_SecurityAccess;

impl UdsService for x27_SecurityAccess {
    const SID: u8 = 0x27;

    type Request<'a> = x27_SecurityAccessRequest<'a>;
    type Response<'a> = x27_SecurityAccessResponse<'a>;
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct x27_SecurityAccessRequest<'a> {
    pub subfunction: UdsSubfunction,
    pub data: &'a [u8],
}

impl<'a> x27_SecurityAccessRequest<'a> {
    pub const MIN_LEN: usize = 1;

    pub const fn encoded_len(&self) -> usize {
        Self::MIN_LEN + self.data.len()
    }

    pub const fn security_access_type(&self) -> Option<SecurityAccessType> {
        SecurityAccessType::from_u8(self.subfunction.parameter_value())
    }

    pub fn security_access_data_record(&self) -> Option<&'a [u8]> {
        self.security_access_type().filter(|t| t.is_request_seed()).map(|_| self.data)
    }

    pub fn security_key(&self) -> Option<&'a [u8]> {
        self.security_access_type().filter(|t| t.is_send_key()).map(|_| self.data)
    }
}

impl<'a> TryFrom<&'a [u8]> for x27_SecurityAccessRequest<'a> {
    type Error = UdsNrc;

    fn try_from(data: &'a [u8]) -> Result<Self, UdsNrc> {
        if data.len() < Self::MIN_LEN {
            return Err(UdsNrc::INCORRECT_MESSAGE_LENGTH_OR_INVALID_FORMAT);
        }
        Ok(Self { subfunction: UdsSubfunction::new(data[0]), data: &data[1..] })
    }
}

impl TryTo for x27_SecurityAccessRequest<'_> {
    type Error = UdsNrc;

    fn try_to(&self, buf: &mut [u8]) -> Result<usize, UdsNrc> {
        let len = self.encoded_len();
        if buf.len() < len {
            return Err(UdsNrc::RESPONSE_TOO_LONG);
        }
        buf[0] = self.subfunction.raw();
        buf[1..len].copy_from_slice(self.data);
        Ok(len)
    }
}

impl<'a> UdsServiceRequest<'a> for x27_SecurityAccessRequest<'a> {
    fn get_subfunction(&self) -> Option<UdsSubfunction> {
        Some(self.subfunction)
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct x27_SecurityAccessResponse<'a> {
    pub subfunction: UdsSubfunction,
    pub security_seed: &'a [u8],
}

impl<'a> x27_SecurityAccessResponse<'a> {
    pub const MIN_LEN: usize = 1;

    pub const fn encoded_len(&self) -> usize {
        Self::MIN_LEN + self.security_seed.len()
    }

    pub const fn security_access_type(&self) -> Option<SecurityAccessType> {
        SecurityAccessType::from_u8(self.subfunction.parameter_value())
    }

    /// According to the spec, a seed of all zeroes means the security level is already unlocked
    pub fn is_already_unlocked(&self) -> bool {
        !self.security_seed.is_empty() && self.security_seed.iter().all(|&b| b == 0)
    }
}

impl<'a> TryFrom<&'a [u8]> for x27_SecurityAccessResponse<'a> {
    type Error = UdsNrc;

    fn try_from(data: &'a [u8]) -> Result<Self, UdsNrc> {
        if data.len() < Self::MIN_LEN {
            return Err(UdsNrc::INCORRECT_MESSAGE_LENGTH_OR_INVALID_FORMAT);
        }
        Ok(Self { subfunction: UdsSubfunction::new(data[0]), security_seed: &data[1..] })
    }
}

impl TryTo for x27_SecurityAccessResponse<'_> {
    type Error = UdsNrc;

    fn try_to(&self, buf: &mut [u8]) -> Result<usize, UdsNrc> {
        let len = self.encoded_len();
        if buf.len() < len {
            return Err(UdsNrc::RESPONSE_TOO_LONG);
        }
        buf[0] = self.subfunction.raw();
        buf[1..len].copy_from_slice(self.security_seed);
        Ok(len)
    }
}

impl<'a> UdsServiceResponse<'a> for x27_SecurityAccessResponse<'a> {}
