use crate::protocol::{frame::Frame, frame_kind::FrameKind, nack_reason::NackReason};

/// What the host asks the device to do. Payloads borrow the receive buffer
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Command<'a> {
    PutScene { json: &'a [u8] },
    PutAsset { id: u16, offset: u32, data: &'a [u8] },
}

impl Command<'_> {
    /// id[2] + offset[4] before the asset data
    pub const PUT_ASSET_HEAD_LEN: usize = 6;
}

impl<'a> TryFrom<Frame<'a>> for Command<'a> {
    type Error = NackReason;

    fn try_from(frame: Frame<'a>) -> Result<Self, Self::Error> {
        match frame.kind {
            FrameKind::PutScene => Ok(Command::PutScene {
                json: frame.payload,
            }),
            FrameKind::PutAsset => {
                let (head, data) = frame
                    .payload
                    .split_at_checked(Self::PUT_ASSET_HEAD_LEN)
                    .ok_or(NackReason::BadPayload)?;
                Ok(Command::PutAsset {
                    id: u16::from_le_bytes([head[0], head[1]]),
                    offset: u32::from_le_bytes([head[2], head[3], head[4], head[5]]),
                    data,
                })
            }
            FrameKind::Ack | FrameKind::Nack => Err(NackReason::UnexpectedKind),
        }
    }
}
