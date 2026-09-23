/// Errors that can occur while decoding or encoding a UDS message or one of its service payloads.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UdsError {
    /// The buffer is shorter than the type's fixed-length fields require.
    TooShort,

    /// The buffer passed to an `encode`/`build` call isn't large enough for the output.
    BufferTooSmall,
}
