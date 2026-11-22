//! Binary framing between the companion (host) and the device.
//!
//! ```text
//! magic[2] kind[1] seq[1] len[2, LE] payload[len] crc16[2, LE]
//! ```
//!
//! CRC is CRC-16/CCITT-FALSE over `kind..=payload`. The device answers every
//! frame with `Ack` or `Nack` carrying the same `seq`.

pub mod command;
pub mod frame;
pub mod frame_kind;
pub mod frame_reader;
pub mod nack_reason;
pub mod response;

#[cfg(test)]
mod tests;
