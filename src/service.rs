/// Static representation of a UDS service and it's requests/responses
pub trait UdsService {
    const SID: u8;

    type Request<'a>;
    type Response<'a>;
}
