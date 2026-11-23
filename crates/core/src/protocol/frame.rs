use crate::protocol::frame_kind::FrameKind;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Frame<'a> {
    pub kind: FrameKind,
    /// number of frame in the sequence, for the future
    pub seq: u8,
    pub payload: &'a [u8],
}

impl<'a> Frame<'a> {
    pub const MAGIC: [u8; 2] = [0x73, 0x78]; // sx magic value
    /// kind + seq + len
    pub const HEADER_LEN: usize = 4;
    pub const CRC_LEN: usize = 2;
    /// Bytes overall
    pub const OVERHEAD: usize = Self::MAGIC.len() + Self::HEADER_LEN + Self::CRC_LEN;

    pub fn new(kind: FrameKind, seq: u8, payload: &'a [u8]) -> Self {
        Self { kind, seq, payload }
    }

    pub fn encode<'o>(&self, out: &'o mut [u8]) -> Option<&'o [u8]> {
        let len = u16::try_from(self.payload.len()).ok()?;
        let total = Self::OVERHEAD + self.payload.len();
        let out = out.get_mut(..total)?;

        let [len_lo, len_hi] = len.to_le_bytes();
        let header = [self.kind as u8, self.seq, len_lo, len_hi];
        let crc_at = Self::MAGIC.len() + Self::HEADER_LEN + self.payload.len();

        out[..Self::MAGIC.len()].copy_from_slice(&Self::MAGIC);
        out[Self::MAGIC.len()..Self::MAGIC.len() + Self::HEADER_LEN].copy_from_slice(&header);
        out[Self::MAGIC.len() + Self::HEADER_LEN..crc_at].copy_from_slice(self.payload);

        let mut crc = Crc16::new();
        crc.update(&out[Self::MAGIC.len()..crc_at]);
        out[crc_at..].copy_from_slice(&crc.finish().to_le_bytes());
        
        Some(out)
    }
}

pub(crate) struct Crc16(u16);

impl Crc16 {
    pub(crate) fn new() -> Self {
        Self(0xFFFF)
    }

    pub(crate) fn update(&mut self, bytes: &[u8]) {
        for &b in bytes {
            self.0 ^= (b as u16) << 8;
            for _ in 0..8 {
                self.0 = if self.0 & 0x8000 != 0 {
                    (self.0 << 1) ^ 0x1021
                } else {
                    self.0 << 1
                };
            }
        }
    }

    pub(crate) fn finish(&self) -> u16 {
        self.0
    }
}
