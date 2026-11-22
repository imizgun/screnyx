extern crate std;

use std::vec::Vec;

use crate::protocol::{
    command::Command,
    frame::{Crc16, Frame},
    frame_kind::FrameKind,
    frame_reader::{FrameReader, Rejected},
    nack_reason::NackReason,
    response::Response,
};

#[derive(Debug, PartialEq)]
enum Got {
    Frame(FrameKind, u8, Vec<u8>),
    Rejected(Rejected),
}

fn encode(kind: u8, seq: u8, payload: &[u8]) -> Vec<u8> {
    // raw kind byte, so tests can send kinds FrameKind does not know
    let mut out = std::vec![0; Frame::OVERHEAD + payload.len()];
    Frame::new(FrameKind::PutScene, seq, payload)
        .encode(&mut out)
        .unwrap();
    
    out[2] = kind;
    let mut crc = Crc16::new();
    crc.update(&out[2..out.len() - Frame::CRC_LEN]);
    let crc_at = out.len() - Frame::CRC_LEN;
    out[crc_at..].copy_from_slice(&crc.finish().to_le_bytes());
    out
}

fn read_all<const N: usize>(reader: &mut FrameReader<N>, chunks: &[&[u8]]) -> Vec<Got> {
    let mut got = Vec::new();
    for chunk in chunks {
        reader.feed(chunk, |result| {
            got.push(match result {
                Ok(frame) => Got::Frame(frame.kind, frame.seq, frame.payload.to_vec()),
                Err(rejected) => Got::Rejected(rejected),
            })
        });
    }
    got
}

#[test]
fn crc_matches_ccitt_false() {
    let mut crc = Crc16::new();
    crc.update(b"123456789");
    assert_eq!(crc.finish(), 0x29B1);
}

#[test]
fn reads_frame_split_byte_by_byte_after_garbage() {
    let mut bytes = std::vec![0x00, 0xA5, 0x13, 0xA5];
    bytes.extend(encode(0x01, 7, b"{\"layers\":[]}"));
    let chunks: Vec<&[u8]> = bytes.chunks(1).collect();

    let got = read_all(&mut FrameReader::<64>::new(), &chunks);
    assert_eq!(
        got,
        [Got::Frame(FrameKind::PutScene, 7, b"{\"layers\":[]}".to_vec())]
    );
}

#[test]
fn reads_empty_payload() {
    let got = read_all(&mut FrameReader::<8>::new(), &[&encode(0x80, 1, &[])]);
    assert_eq!(got, [Got::Frame(FrameKind::Ack, 1, Vec::new())]);
}

#[test]
fn rejects_and_recovers() {
    let mut bad_crc = encode(0x01, 1, b"abc");
    *bad_crc.last_mut().unwrap() ^= 0xFF;
    let too_long = encode(0x01, 2, &[0; 9]);
    let unknown = encode(0x7F, 3, b"xyz");
    let good = encode(0x01, 4, b"ok");

    let got = read_all(
        &mut FrameReader::<8>::new(),
        &[&bad_crc, &too_long, &unknown, &good],
    );
    let rejected = |seq, reason| Got::Rejected(Rejected { seq, reason });
    assert_eq!(
        got,
        [
            rejected(1, NackReason::BadCrc),
            rejected(2, NackReason::PayloadTooLong),
            rejected(3, NackReason::UnknownKind),
            Got::Frame(FrameKind::PutScene, 4, b"ok".to_vec()),
        ]
    );
}

#[test]
fn reset_drops_partial_frame() {
    let mut reader = FrameReader::<8>::new();
    let frame = encode(0x01, 1, b"abc");
    read_all(&mut reader, &[&frame[..5]]);
    reader.reset();
    let got = read_all(&mut reader, &[&encode(0x01, 2, b"ok")]);
    assert_eq!(got, [Got::Frame(FrameKind::PutScene, 2, b"ok".to_vec())]);
}

#[test]
fn parses_commands() {
    let scene = Frame::new(FrameKind::PutScene, 0, b"{}");
    assert_eq!(
        Command::try_from(scene),
        Ok(Command::PutScene { json: b"{}" })
    );

    let asset = [0x34, 0x12, 0x00, 0x01, 0x00, 0x00, 0xAA, 0xBB];
    assert_eq!(
        Command::try_from(Frame::new(FrameKind::PutAsset, 0, &asset)),
        Ok(Command::PutAsset {
            id: 0x1234,
            offset: 0x100,
            data: &[0xAA, 0xBB],
        })
    );

    let short = Frame::new(FrameKind::PutAsset, 0, &asset[..5]);
    assert_eq!(Command::try_from(short), Err(NackReason::BadPayload));

    let ack = Frame::new(FrameKind::Ack, 0, &[]);
    assert_eq!(Command::try_from(ack), Err(NackReason::UnexpectedKind));
}

#[test]
fn response_roundtrips_through_reader() {
    let mut out = [0; Response::MAX_LEN];
    let nack = Response::from(Rejected {
        seq: 9,
        reason: NackReason::BadScene,
    });

    let got = read_all(&mut FrameReader::<1>::new(), &[nack.encode(&mut out)]);
    assert_eq!(
        got,
        [Got::Frame(FrameKind::Nack, 9, std::vec![NackReason::BadScene as u8])]
    );
}
