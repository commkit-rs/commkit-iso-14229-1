/// A UDS sub-function byte: the top bit is `suppressPositiveResponseMessageIndicationBit`, the
/// bottom 7 bits are the sub-function value itself.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UdsSubfunction(u8);

impl UdsSubfunction {
    pub const fn new(raw: u8) -> Self {
        Self(raw)
    }

    pub const fn from_parts(value: u8, suppress_positive_response: bool) -> Self {
        let suppress_bit = if suppress_positive_response { 0x80 } else { 0x00 };
        Self((value & 0x7F) | suppress_bit)
    }

    pub const fn raw(self) -> u8 {
        self.0
    }

    /// Whether the requester asked the server not to send a positive response.
    pub const fn suppress_pos_rsp_msg_indication_bit(self) -> bool {
        self.0 & 0x80 != 0
    }

    /// The sub-function value, with the suppress bit masked off.
    pub const fn parameter_value(self) -> u8 {
        self.0 & 0x7F
    }
}
