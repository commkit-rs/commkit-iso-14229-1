#![no_std]

mod client;
mod memory;
mod message;
mod nrc;
mod server;
mod service;
mod sid;
mod subfunction;

pub mod services;

pub use client::{UdsClient, UdsClientConfig, UdsClientError, UdsClientEvent};
pub use memory::{
    AddressAndLengthFormatIdentifier, DataFormatIdentifier, LengthFormatIdentifier, MaxNumberOfBlockLength,
    MemoryAddressAndSize,
};
pub use message::UdsMessage;
pub use nrc::UdsNrc;
pub use service::{UdsService, UdsServiceRequest, UdsServiceResponse};
pub use server::{
    DispatchResult, HandlerResult, SECURITY_LEVEL_LOCKED, UdsRequestHandler, UdsSendFn, UdsServer, UdsServerConfig,
    UdsServerEntry, UdsServerOutcome, UdsServerState, UdsServerTable, positive_response_sid,
};
pub use sid::UdsSid;
pub use subfunction::UdsSubfunction;

/// UDS protocol version, packed as `major << 24 | minor << 16 | revision << 8 | b4` as defined in the spec
pub const UDS_PROTOCOL_VERSION: u32 = (2 << 24) | (0 << 16) | (0 << 8) | 0;

/// Service byte offset for positive acknowledgement
pub const UDS_PROTOCOL_POSITIVE_RESPONSE_SID_OFFSET: u8 = 0x40;
