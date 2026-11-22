/// Why the device rejected a frame, sent as the `Nack` payload.
/// Never reuse or renumber a published value: add a new one instead.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NackReason {
    /// `kind` is not a known `FrameKind`
    UnknownKind = 0x01,
    /// `kind` is known but not meant for the device, e.g. `Ack`
    UnexpectedKind = 0x02,
    /// `len` is over the receiver buffer
    PayloadTooLong = 0x03,
    BadCrc = 0x04,
    /// payload is shorter than the fixed fields of its kind
    BadPayload = 0x05,
    /// `PutScene` JSON did not parse into a `Scene`
    BadScene = 0x06,
}
