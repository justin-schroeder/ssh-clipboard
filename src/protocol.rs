use std::io;

use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use uuid::Uuid;

use crate::model::{Clip, PROTOCOL_VERSION, Representation};

const MAGIC: [u8; 4] = *b"SCB1";
const MAX_HEADER_BYTES: u32 = 1024 * 1024;

#[derive(Debug, Error)]
pub enum ProtocolError {
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),
    #[error("invalid protocol magic")]
    InvalidMagic,
    #[error("invalid header length {0}")]
    InvalidHeaderLength(u32),
    #[error("invalid protocol header: {0}")]
    InvalidHeader(#[from] serde_json::Error),
    #[error("unsupported protocol version {0}")]
    UnsupportedVersion(u16),
    #[error("payload exceeds {0} byte limit")]
    PayloadTooLarge(u64),
    #[error("invalid message: {0}")]
    InvalidMessage(&'static str),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Message {
    Hello { node_id: Uuid, node_name: String },
    Clip(Clip),
}

#[derive(Serialize, Deserialize)]
struct Header {
    version: u16,
    kind: Kind,
    #[serde(skip_serializing_if = "Option::is_none")]
    node_id: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    node_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    clip_id: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    origin: Option<Uuid>,
    #[serde(default)]
    created_millis: u64,
    #[serde(default)]
    representations: Vec<RepresentationHeader>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Kind {
    Hello,
    Clip,
}

#[derive(Serialize, Deserialize)]
struct RepresentationHeader {
    item: u32,
    format: String,
    size: u64,
}

pub async fn write_message<W>(writer: &mut W, message: &Message, max_bytes: u64) -> Result<(), ProtocolError>
where
    W: AsyncWrite + Unpin,
{
    let (header, data): (Header, Vec<&[u8]>) = match message {
        Message::Hello { node_id, node_name } => (
            Header {
                version: PROTOCOL_VERSION,
                kind: Kind::Hello,
                node_id: Some(*node_id),
                node_name: Some(node_name.clone()),
                clip_id: None,
                origin: None,
                created_millis: 0,
                representations: Vec::new(),
            },
            Vec::new(),
        ),
        Message::Clip(clip) => return write_clip(writer, clip, max_bytes).await,
    };
    write_parts(writer, &header, &data).await
}

pub async fn write_clip<W>(writer: &mut W, clip: &Clip, max_bytes: u64) -> Result<(), ProtocolError>
where
    W: AsyncWrite + Unpin,
{
    if clip.total_bytes() > max_bytes {
        return Err(ProtocolError::PayloadTooLarge(max_bytes));
    }
    let header = Header {
        version: PROTOCOL_VERSION,
        kind: Kind::Clip,
        node_id: None,
        node_name: None,
        clip_id: Some(clip.id),
        origin: Some(clip.origin),
        created_millis: clip.created_millis,
        representations: clip
            .representations
            .iter()
            .map(|representation| RepresentationHeader {
                item: representation.item,
                format: representation.format.clone(),
                size: u64::try_from(representation.data.len()).unwrap_or(u64::MAX),
            })
            .collect(),
    };
    let data = clip
        .representations
        .iter()
        .map(|representation| representation.data.as_slice())
        .collect::<Vec<_>>();
    write_parts(writer, &header, &data).await
}

async fn write_parts<W>(writer: &mut W, header: &Header, data: &[&[u8]]) -> Result<(), ProtocolError>
where
    W: AsyncWrite + Unpin,
{
    let encoded = serde_json::to_vec(header)?;
    let header_len =
        u32::try_from(encoded.len()).map_err(|_| ProtocolError::InvalidHeaderLength(u32::MAX))?;
    if header_len == 0 || header_len > MAX_HEADER_BYTES {
        return Err(ProtocolError::InvalidHeaderLength(header_len));
    }
    writer.write_all(&MAGIC).await?;
    writer.write_u32(header_len).await?;
    writer.write_all(&encoded).await?;
    for bytes in data {
        writer.write_all(bytes).await?;
    }
    writer.flush().await?;
    Ok(())
}

pub async fn read_message<R>(reader: &mut R, max_bytes: u64) -> Result<Message, ProtocolError>
where
    R: AsyncRead + Unpin,
{
    let mut magic = [0; 4];
    reader.read_exact(&mut magic).await?;
    if magic != MAGIC {
        return Err(ProtocolError::InvalidMagic);
    }
    let header_len = reader.read_u32().await?;
    if header_len == 0 || header_len > MAX_HEADER_BYTES {
        return Err(ProtocolError::InvalidHeaderLength(header_len));
    }
    let mut encoded = vec![0; header_len as usize];
    reader.read_exact(&mut encoded).await?;
    let header: Header = serde_json::from_slice(&encoded)?;
    if header.version != PROTOCOL_VERSION {
        return Err(ProtocolError::UnsupportedVersion(header.version));
    }
    match header.kind {
        Kind::Hello => Ok(Message::Hello {
            node_id: header
                .node_id
                .ok_or(ProtocolError::InvalidMessage("hello is missing node_id"))?,
            node_name: header
                .node_name
                .filter(|name| !name.is_empty())
                .ok_or(ProtocolError::InvalidMessage("hello is missing node_name"))?,
        }),
        Kind::Clip => {
            let mut total = 0_u64;
            let mut representations = Vec::with_capacity(header.representations.len());
            for representation in header.representations {
                total = total
                    .checked_add(representation.size)
                    .ok_or(ProtocolError::PayloadTooLarge(max_bytes))?;
                if total > max_bytes {
                    return Err(ProtocolError::PayloadTooLarge(max_bytes));
                }
                let size = usize::try_from(representation.size)
                    .map_err(|_| ProtocolError::PayloadTooLarge(max_bytes))?;
                let mut data = vec![0; size];
                reader.read_exact(&mut data).await?;
                representations.push(Representation {
                    item: representation.item,
                    format: representation.format,
                    data,
                });
            }
            Ok(Message::Clip(Clip {
                id: header
                    .clip_id
                    .ok_or(ProtocolError::InvalidMessage("clip is missing clip_id"))?,
                origin: header
                    .origin
                    .ok_or(ProtocolError::InvalidMessage("clip is missing origin"))?,
                created_millis: header.created_millis,
                representations,
            }))
        }
    }
}

#[cfg(test)]
mod tests {
    use tokio::io::duplex;

    use super::*;

    #[tokio::test]
    async fn round_trips_multiple_native_formats_without_base64() {
        let clip = Clip::new(
            Uuid::new_v4(),
            vec![
                Representation {
                    item: 0,
                    format: "public.tiff".into(),
                    data: vec![0, 1, 2, 255],
                },
                Representation {
                    item: 0,
                    format: "text/html".into(),
                    data: b"<b>hello</b>".to_vec(),
                },
            ],
        );
        let expected = Message::Clip(clip);
        let (mut left, mut right) = duplex(4096);
        let write = write_message(&mut left, &expected, 1024);
        let read = read_message(&mut right, 1024);
        let (written, actual) = tokio::join!(write, read);
        written.unwrap();
        assert_eq!(actual.unwrap(), expected);
    }

    #[tokio::test]
    async fn rejects_payload_over_limit_before_writing() {
        let clip = Clip::new(
            Uuid::nil(),
            vec![Representation {
                item: 0,
                format: "application/octet-stream".into(),
                data: vec![0; 17],
            }],
        );
        let (mut writer, _reader) = duplex(128);
        assert!(matches!(
            write_message(&mut writer, &Message::Clip(clip), 16).await,
            Err(ProtocolError::PayloadTooLarge(16))
        ));
    }
}
