use crate::protocol::{
    frame::Frame, frame_kind::FrameKind, frame_reader::Rejected, nack_reason::NackReason,
};

/// The device's answer to a sent frame
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Response {
    Ack { seq: u8 },
    Nack { seq: u8, reason: NackReason },
}

impl Response {
    /// The longest response on the wire: `Nack` with its reason byte
    pub const MAX_LEN: usize = Frame::OVERHEAD + 1;

    pub fn encode<'o>(&self, out: &'o mut [u8; Self::MAX_LEN]) -> &'o [u8] {
        let reason_byte;
        let frame = match *self {
            Response::Ack { seq } => Frame::new(FrameKind::Ack, seq, &[]),
            Response::Nack { seq, reason } => {
                reason_byte = [reason as u8];
                Frame::new(FrameKind::Nack, seq, &reason_byte)
            }
        };
        frame.encode(out)
            .expect("Response::MAX_LEN fits every response")
    }
}

impl From<Rejected> for Response {
    fn from(rejected: Rejected) -> Self {
        Response::Nack {
            seq: rejected.seq,
            reason: rejected.reason,
        }
    }
}
