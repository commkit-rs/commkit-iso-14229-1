use commkit::{Direction, Message};

use crate::nrc::UdsNrc;

pub struct UdsMessage<M: Message> {
    inner: M,
}

impl<M: Message> UdsMessage<M> {
    pub fn parse(inner: M) -> Result<Self, UdsNrc> {
        if inner.as_bytes().is_empty() {
            return Err(UdsNrc::INCORRECT_MESSAGE_LENGTH_OR_INVALID_FORMAT);
        }
        Ok(Self { inner })
    }

    /// Writes `sid` followed by `payload` into `scratch` and wraps the result as `M`.
    pub fn build(direction: Direction, sid: u8, payload: &[u8], scratch: &mut [u8]) -> Result<Self, UdsNrc> {
        let len = 1 + payload.len();
        if scratch.len() < len {
            return Err(UdsNrc::RESPONSE_TOO_LONG);
        }

        scratch[0] = sid;
        scratch[1..len].copy_from_slice(payload);

        Ok(Self { inner: M::new(direction, &scratch[..len]) })
    }

    pub fn service_id(&self) -> u8 {
        self.inner.as_bytes()[0]
    }

    /// The bytes after the SID.
    pub fn payload(&self) -> &[u8] {
        &self.inner.as_bytes()[1..]
    }

    pub fn into_inner(self) -> M {
        self.inner
    }
}
