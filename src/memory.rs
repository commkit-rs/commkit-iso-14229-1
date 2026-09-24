use crate::nrc::UdsNrc;

const MAX_FIELD_LEN: u8 = 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AddressAndLengthFormatIdentifier(u8);

impl AddressAndLengthFormatIdentifier {
    pub const LEN: usize = 1;

    pub const fn new(raw: u8) -> Self {
        Self(raw)
    }

    pub const fn from_parts(memory_address_len: u8, memory_size_len: u8) -> Self {
        Self(((memory_size_len & 0x0F) << 4) | (memory_address_len & 0x0F))
    }

    pub const fn minimal(memory_address: u64, memory_size: u64) -> Self {
        Self::from_parts(min_bytes(memory_address), min_bytes(memory_size))
    }

    pub const fn raw(self) -> u8 {
        self.0
    }

    pub const fn memory_address_len(self) -> u8 {
        self.0 & 0x0F
    }

    pub const fn memory_size_len(self) -> u8 {
        self.0 >> 4
    }

    pub const fn is_valid(self) -> bool {
        let address_len = self.memory_address_len();
        let size_len = self.memory_size_len();
        address_len != 0 && address_len <= MAX_FIELD_LEN && size_len != 0 && size_len <= MAX_FIELD_LEN
    }

    pub const fn fields_len(self) -> usize {
        self.memory_address_len() as usize + self.memory_size_len() as usize
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MemoryAddressAndSize {
    pub address_and_length_format_identifier: AddressAndLengthFormatIdentifier,
    pub memory_address: u64,
    pub memory_size: u64,
}

impl MemoryAddressAndSize {
    pub const MIN_LEN: usize = AddressAndLengthFormatIdentifier::LEN + 2;

    pub const fn new(memory_address: u64, memory_size: u64) -> Self {
        Self {
            address_and_length_format_identifier: AddressAndLengthFormatIdentifier::minimal(memory_address, memory_size),
            memory_address,
            memory_size,
        }
    }

    pub fn decode(data: &[u8]) -> Result<(Self, usize), UdsNrc> {
        if data.len() < Self::MIN_LEN {
            return Err(UdsNrc::INCORRECT_MESSAGE_LENGTH_OR_INVALID_FORMAT);
        }
        let format = AddressAndLengthFormatIdentifier::new(data[0]);
        if !format.is_valid() {
            return Err(UdsNrc::REQUEST_OUT_OF_RANGE);
        }
        let len = AddressAndLengthFormatIdentifier::LEN + format.fields_len();
        if data.len() < len {
            return Err(UdsNrc::INCORRECT_MESSAGE_LENGTH_OR_INVALID_FORMAT);
        }
        let address_end = AddressAndLengthFormatIdentifier::LEN + format.memory_address_len() as usize;
        Ok((
            Self {
                address_and_length_format_identifier: format,
                memory_address: read_be(&data[AddressAndLengthFormatIdentifier::LEN..address_end]),
                memory_size: read_be(&data[address_end..len]),
            },
            len,
        ))
    }

    pub fn encode(&self, buf: &mut [u8]) -> Result<usize, UdsNrc> {
        let format = self.address_and_length_format_identifier;
        if !format.is_valid()
            || min_bytes(self.memory_address) > format.memory_address_len()
            || min_bytes(self.memory_size) > format.memory_size_len()
        {
            return Err(UdsNrc::REQUEST_OUT_OF_RANGE);
        }
        let len = self.encoded_len();
        if buf.len() < len {
            return Err(UdsNrc::RESPONSE_TOO_LONG);
        }
        let address_end = AddressAndLengthFormatIdentifier::LEN + format.memory_address_len() as usize;
        buf[0] = format.raw();
        write_be(self.memory_address, &mut buf[AddressAndLengthFormatIdentifier::LEN..address_end]);
        write_be(self.memory_size, &mut buf[address_end..len]);
        Ok(len)
    }

    pub const fn encoded_len(&self) -> usize {
        AddressAndLengthFormatIdentifier::LEN + self.address_and_length_format_identifier.fields_len()
    }
}

const fn min_bytes(value: u64) -> u8 {
    let bytes = (u64::BITS - value.leading_zeros()).div_ceil(8) as u8;
    if bytes == 0 { 1 } else { bytes }
}

fn read_be(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0, |acc, &b| (acc << 8) | b as u64)
}

fn write_be(value: u64, out: &mut [u8]) {
    let be = value.to_be_bytes();
    out.copy_from_slice(&be[be.len() - out.len()..]);
}
