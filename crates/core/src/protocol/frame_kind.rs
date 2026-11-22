use crate::protocol::nack_reason::NackReason;

/// Frame type byte. Never reuse or renumber a published value: add a new one instead
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FrameKind {
    PutScene = 0x01,
    PutAsset = 0x02,
    Ack = 0x80,
    Nack = 0x81,
}

impl TryFrom<u8> for FrameKind {
    type Error = NackReason;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x01 => Ok(Self::PutScene),
            0x02 => Ok(Self::PutAsset),
            0x80 => Ok(Self::Ack),
            0x81 => Ok(Self::Nack),
            _ => Err(NackReason::UnknownKind),
        }
    }
}
