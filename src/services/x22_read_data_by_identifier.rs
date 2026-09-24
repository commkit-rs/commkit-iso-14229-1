use crate::nrc::UdsNrc;
use crate::service::UdsService;

/*
    ISO 14229-1 Section 10.2

    The ReadDataByIdentifier service is used to request data record values from the server identified by one or more dataIdentifiers

    Supported NRC:
        - IMLOIF
        - RTL
        - CNC
        - ROOR
        - SAD
*/

const DID_LEN: usize = 2;

#[allow(non_camel_case_types)]
pub struct x22_ReadDataByIdentifier;

impl UdsService for x22_ReadDataByIdentifier {
    const SID: u8 = 0x22;

    type Request<'a> = x22_ReadDataByIdentifierRequest<'a>;
    type Response<'a> = x22_ReadDataByIdentifierResponse<'a>;
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct x22_ReadDataByIdentifierRequest<'a> {
    data: &'a [u8],
}

impl<'a> x22_ReadDataByIdentifierRequest<'a> {
    pub fn decode(data: &'a [u8]) -> Result<Self, UdsNrc> {
        if data.is_empty() || !data.len().is_multiple_of(DID_LEN) {
            return Err(UdsNrc::INCORRECT_MESSAGE_LENGTH_OR_INVALID_FORMAT);
        }
        Ok(Self { data })
    }

    pub fn encode_did(did: u16, buf: &mut [u8]) -> Result<usize, UdsNrc> {
        Self::encode_dids(&[did], buf)
    }

    pub fn encode_dids(dids: &[u16], buf: &mut [u8]) -> Result<usize, UdsNrc> {
        let len = dids.len() * DID_LEN;
        if dids.is_empty() {
            return Err(UdsNrc::INCORRECT_MESSAGE_LENGTH_OR_INVALID_FORMAT);
        }
        if buf.len() < len {
            return Err(UdsNrc::RESPONSE_TOO_LONG);
        }
        for (chunk, did) in buf.chunks_exact_mut(DID_LEN).zip(dids) {
            chunk.copy_from_slice(&did.to_be_bytes());
        }
        Ok(len)
    }

    pub fn dids(&self) -> impl Iterator<Item = u16> + 'a {
        self.data.chunks_exact(DID_LEN).map(|chunk| u16::from_be_bytes([chunk[0], chunk[1]]))
    }

    pub const fn did_count(&self) -> usize {
        self.data.len() / DID_LEN
    }

    pub const fn as_bytes(&self) -> &'a [u8] {
        self.data
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct x22_ReadDataByIdentifierResponse<'a> {
    data: &'a [u8],
}

impl<'a> x22_ReadDataByIdentifierResponse<'a> {
    pub fn decode(data: &'a [u8]) -> Result<Self, UdsNrc> {
        if data.len() < DID_LEN {
            return Err(UdsNrc::INCORRECT_MESSAGE_LENGTH_OR_INVALID_FORMAT);
        }
        Ok(Self { data })
    }

    pub fn encode_single(did: u16, value: &[u8], buf: &mut [u8]) -> Result<usize, UdsNrc> {
        let mut writer = x22_ReadDataByIdentifierResponseWriter::new(buf);
        writer.push(did, value)?;
        Ok(writer.finish())
    }

    pub fn single(&self) -> (u16, &'a [u8]) {
        (u16::from_be_bytes([self.data[0], self.data[1]]), &self.data[DID_LEN..])
    }

    pub fn records<F: Fn(u16) -> Option<usize>>(&self, len_of: F) -> x22_ReadDataByIdentifierRecords<'a, F> {
        x22_ReadDataByIdentifierRecords { remaining: self.data, len_of }
    }

    pub const fn as_bytes(&self) -> &'a [u8] {
        self.data
    }
}

#[allow(non_camel_case_types)]
pub struct x22_ReadDataByIdentifierRecords<'a, F> {
    remaining: &'a [u8],
    len_of: F,
}

impl<'a, F: Fn(u16) -> Option<usize>> Iterator for x22_ReadDataByIdentifierRecords<'a, F> {
    type Item = Result<(u16, &'a [u8]), UdsNrc>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining.is_empty() {
            return None;
        }

        let record = self.next_record();
        if record.is_err() {
            self.remaining = &[];
        }
        Some(record)
    }
}

impl<'a, F: Fn(u16) -> Option<usize>> x22_ReadDataByIdentifierRecords<'a, F> {
    fn next_record(&mut self) -> Result<(u16, &'a [u8]), UdsNrc> {
        if self.remaining.len() < DID_LEN {
            return Err(UdsNrc::INCORRECT_MESSAGE_LENGTH_OR_INVALID_FORMAT);
        }
        let did = u16::from_be_bytes([self.remaining[0], self.remaining[1]]);
        let len = (self.len_of)(did).ok_or(UdsNrc::INCORRECT_MESSAGE_LENGTH_OR_INVALID_FORMAT)?;
        let end = DID_LEN + len;
        if self.remaining.len() < end {
            return Err(UdsNrc::INCORRECT_MESSAGE_LENGTH_OR_INVALID_FORMAT);
        }
        let value = &self.remaining[DID_LEN..end];
        self.remaining = &self.remaining[end..];
        Ok((did, value))
    }
}

#[allow(non_camel_case_types)]
pub struct x22_ReadDataByIdentifierResponseWriter<'b> {
    buf: &'b mut [u8],
    len: usize,
}

impl<'b> x22_ReadDataByIdentifierResponseWriter<'b> {
    pub fn new(buf: &'b mut [u8]) -> Self {
        Self { buf, len: 0 }
    }

    pub fn push(&mut self, did: u16, value: &[u8]) -> Result<(), UdsNrc> {
        let end = self.len + DID_LEN + value.len();
        if self.buf.len() < end {
            return Err(UdsNrc::RESPONSE_TOO_LONG);
        }
        self.buf[self.len..self.len + DID_LEN].copy_from_slice(&did.to_be_bytes());
        self.buf[self.len + DID_LEN..end].copy_from_slice(value);
        self.len = end;
        Ok(())
    }

    pub const fn len(&self) -> usize {
        self.len
    }

    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn finish(self) -> usize {
        self.len
    }
}
