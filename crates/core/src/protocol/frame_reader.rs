use heapless::Vec;

use crate::protocol::{
    frame::{Crc16, Frame},
    frame_kind::FrameKind,
    nack_reason::NackReason,
};

/// Not accepted frame information, goes with `Nack`
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rejected {
    pub seq: u8,
    pub reason: NackReason,
}

#[derive(Clone, Copy)]
struct Header {
    kind: FrameKind,
    seq: u8,
    len: usize,
}

/// State machine for frame reading
#[derive(Clone, Copy)]
enum State {
    /// Looking for `Frame::MAGIC`, holds the number of magic bytes matched so far
    Magic(usize),
    /// Collecting kind, seq and len, holds the number of header bytes got so far
    Header(usize),
    /// Collecting the payload, `len` is known to fit into the buffer
    Payload(Header),
    /// Collecting the CRC, holds the number of CRC bytes got so far
    Crc(Header, usize),
    /// Dropping a frame that cannot be accepted, `remaining` counts its payload and CRC
    Skip {
        seq: u8,
        reason: NackReason,
        remaining: usize,
    },
}

/// Assembles frames from a byte stream. `N` is the largest payload accepted.
pub struct FrameReader<const N: usize> {
    state: State,
    header: [u8; Frame::HEADER_LEN],
    crc: [u8; Frame::CRC_LEN],
    payload: Vec<u8, N>,
}

impl<const N: usize> FrameReader<N> {
    pub fn new() -> Self {
        Self {
            state: State::Magic(0),
            header: [0; Frame::HEADER_LEN],
            crc: [0; Frame::CRC_LEN],
            payload: Vec::new(),
        }
    }

    /// Drops a partially received frame
    pub fn reset(&mut self) {
        self.state = State::Magic(0);
    }

    pub fn feed(&mut self, bytes: &[u8], mut on_frame: impl FnMut(Result<Frame<'_>, Rejected>)) {
        for &b in bytes {
            self.state = match self.state {
                // on first found matched is always 0
                State::Magic(matched) => {
                    if b == Frame::MAGIC[matched] {
                        if matched + 1 == Frame::MAGIC.len() { // if there're all magic bytes collected, going to header state
                            State::Header(0)
                        } else {
                            State::Magic(matched + 1)
                        }
                    } else if b == Frame::MAGIC[0] {
                        State::Magic(1)
                    } else {
                        State::Magic(0)
                    }
                },
                State::Header(got) => {
                    self.header[got] = b;
                    if got + 1 < Frame::HEADER_LEN {
                        State::Header(got + 1)
                    } else {
                        self.parse_header()
                    }
                },
                State::Payload(header) => {
                    // never fails: parse_header checked that `len` fits into `N`
                    let _ = self.payload.push(b);
                    if self.payload.len() < header.len {
                        State::Payload(header)
                    } else {
                        State::Crc(header, 0)
                    }
                },
                State::Crc(header, got) => {
                    self.crc[got] = b;
                    if got + 1 < Frame::CRC_LEN {
                        State::Crc(header, got + 1)
                    } else {
                        on_frame(self.check(header));
                        State::Magic(0)
                    }
                }
                State::Skip {
                    seq,
                    reason,
                    remaining,
                } => {
                    if remaining > 1 {
                        State::Skip {
                            seq,
                            reason,
                            remaining: remaining - 1,
                        }
                    } else {
                        on_frame(Err(Rejected { seq, reason }));
                        State::Magic(0)
                    }
                }
            }
        }
    }

    fn parse_header(&mut self) -> State {
        let [kind, seq, len_lo, len_hi] = self.header;
        let len = u16::from_le_bytes([len_lo, len_hi]) as usize;

        let skip = |reason| State::Skip {
            seq,
            reason,
            remaining: len + Frame::CRC_LEN,
        };

        let kind = match FrameKind::try_from(kind) {
            Ok(kind) => kind,
            Err(reason) => return skip(reason), // unknown reason
        };
        
        if len > N {
            return skip(NackReason::PayloadTooLong);
        }

        self.payload.clear();
        
        let header = Header { kind, seq, len };

        if len == 0 {
            State::Crc(header, 0)
        } else {
            State::Payload(header)
        }
    }

    fn check(&self, header: Header) -> Result<Frame<'_>, Rejected> {
        let mut crc = Crc16::new();
        
        crc.update(&self.header);
        crc.update(&self.payload);

        if crc.finish() == u16::from_le_bytes(self.crc) {
            Ok(Frame::new(header.kind, header.seq, &self.payload))
        } else {
            Err(Rejected {
                seq: header.seq,
                reason: NackReason::BadCrc,
            })
        }
    }
}
