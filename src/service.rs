use commkit::TryTo;

use crate::nrc::UdsNrc;
use crate::subfunction::UdsSubfunction;

/// Static representation of a UDS service and it's requests/responses
pub trait UdsService {
    const SID: u8;

    type Request<'a>: UdsServiceRequest<'a>;
    type Response<'a>: UdsServiceResponse<'a>;
}

/// A UDS service request payload (everything after the SID)
pub trait UdsServiceRequest<'a>: TryFrom<&'a [u8], Error = UdsNrc> + TryTo<Error = UdsNrc> {
    fn get_subfunction(&self) -> Option<UdsSubfunction>;
}

/// A UDS service positive response payload (everything after the response SID)
pub trait UdsServiceResponse<'a>: TryFrom<&'a [u8], Error = UdsNrc> + TryTo<Error = UdsNrc> {}
