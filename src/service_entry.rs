use crate::service::UdsService;

/// For use by UDS dispatchers to bind a static UdsService description to an actual implementation
pub struct UdsServiceEntry<S: UdsService + 'static> {
    pub service: &'static S,
}

impl<S: UdsService + 'static> UdsServiceEntry<S> {
    pub const fn new(service: &'static S) -> Self {
        Self { service }
    }

    pub fn sid(&self) -> u8 {
        S::SID
    }
}
