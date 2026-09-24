#![no_std]

mod memory;
mod message;
mod nrc;
mod service;
mod service_entry;
mod sid;
mod subfunction;

pub mod services;

pub use memory::{AddressAndLengthFormatIdentifier, MemoryAddressAndSize};
pub use message::{UdsMessage, UdsNegativeResponse};
pub use nrc::UdsNrc;
pub use service::UdsService;
pub use service_entry::UdsServiceEntry;
pub use sid::UdsSid;
pub use subfunction::UdsSubfunction;

/// UDS protocol version, packed as `major << 24 | minor << 16 | revision << 8 | b4` as defined in the spec
pub const UDS_PROTOCOL_VERSION: u32 = (2 << 24) | (0 << 16) | (0 << 8) | 0;

/// Service byte offset for positive acknowledgement
pub const UDS_PROTOCOL_POSITIVE_RESPONSE_SID_OFFSET: u8 = 0x40;
