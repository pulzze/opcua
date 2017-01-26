use std::io::{Read, Write, Cursor, Result, Error, ErrorKind};

use opcua_core::types::*;

const HELLO_MESSAGE: &'static [u8] = b"HEL";
const ACKNOWLEDGE_MESSAGE: &'static [u8] = b"ACK";
const ERROR_MESSAGE: &'static [u8] = b"ERR";

#[derive(Debug, Clone, PartialEq)]
pub enum MessageType {
    Invalid,
    Hello,
    Acknowledge,
    Error
}

#[derive(Debug, Clone, PartialEq)]
pub struct MessageHeader {
    pub message_type: MessageType,
    pub message_size: UInt32,
}

impl BinaryEncoder<MessageHeader> for MessageHeader {
    fn byte_len(&self) -> usize {
        8
    }

    fn encode<S: Write>(&self, stream: &mut S) -> EncodingResult<usize> {
        let mut size: usize = 0;
        let result = match self.message_type {
            MessageType::Hello => {
                stream.write(HELLO_MESSAGE)
            },
            MessageType::Acknowledge => {
                stream.write(ACKNOWLEDGE_MESSAGE)
            }
            MessageType::Error => {
                stream.write(ERROR_MESSAGE)
            }
            _ => {
                panic!("Unrecognized type");
            }
        };
        size += process_encode_io_result(result)?;
        size += write_u8(stream, b'F')?;
        size += write_u32(stream, self.message_size)?;
        Ok(size)
    }

    fn decode<S: Read>(stream: &mut S) -> EncodingResult<Self> {
        let mut message_type: [u8; 4] = [0, 0, 0, 0];
        process_decode_io_result(stream.read_exact(&mut message_type))?;
        let message_size = read_u32(stream)?;
        Ok(MessageHeader {
            message_type: MessageHeader::message_type(&message_type),
            message_size: message_size,
        })
    }
}

impl MessageHeader {
    pub fn new(message_type: MessageType) -> MessageHeader {
        MessageHeader {
            message_type: message_type,
            message_size: 0,
        }
    }

    /// Reads the bytes of the stream to a buffer. If first 4 bytes are invalid,
    /// code returns an error
    pub fn read_bytes<S: Read>(stream: &mut S) -> Result<Vec<u8>> {
        // Read the bytes of the stream into a vector
        let mut header: [u8; 4] = [0u8; 4];
        stream.read_exact(&mut header)?;
        if MessageHeader::message_type(&header) == MessageType::Invalid {
            return Err(Error::new(ErrorKind::Other, "Message type is not recognized, cannot read bytes"))
        }
        let message_size = UInt32::decode(stream);
        if message_size.is_err() {
            return Err(Error::new(ErrorKind::Other, "Cannot decode message_size"));
        }
        let message_size = message_size.unwrap();

        // Write header to stream
        let mut out = Cursor::new(Vec::with_capacity(message_size as usize));
        let result = out.write(&header);
        if result.is_err() {
            return Err(Error::new(ErrorKind::Other, "Cannot write message header to buffer "));
        }

        let result = message_size.encode(&mut out);
        if result.is_err() {
            return Err(Error::new(ErrorKind::Other, "Cannot write message size to buffer "));
        }

        let pos = out.position() as usize;
        // Read remaining bytes straight into the vec
        let mut result = out.into_inner();
        result.resize(message_size as usize, 0u8);
        stream.read_exact(&mut result[pos..])?;

        Ok(result)
    }

    pub fn message_type(t: &[u8]) -> MessageType {
        println!("Message type input = {:?}", t);
        if t.len() != 4 {
            println!("Message type len != 4");
            MessageType::Invalid
        } else if t[3] != b'F' {
            println!("Message 4th byte is not F");
            MessageType::Invalid
        } else {
            match &t[0..3] {
                HELLO_MESSAGE => MessageType::Hello,
                ACKNOWLEDGE_MESSAGE => MessageType::Acknowledge,
                ERROR_MESSAGE => MessageType::Error,
                _ => {
                    error!("message type doesn't match anything");
                    MessageType::Invalid
                },
            }
        }
    }
}


#[derive(Debug, Clone, PartialEq)]
pub struct HelloMessage {
    pub message_header: MessageHeader,
    pub protocol_version: UInt32,
    pub receive_buffer_size: UInt32,
    pub send_buffer_size: UInt32,
    pub max_message_size: UInt32,
    pub max_chunk_count: UInt32,
    pub endpoint_url: UAString,
}

impl BinaryEncoder<HelloMessage> for HelloMessage {
    fn byte_len(&self) -> usize {
        // 5 * u32 = 20
        self.message_header.byte_len() + 20 + self.endpoint_url.byte_len()
    }

    fn encode<S: Write>(&self, stream: &mut S) -> EncodingResult<usize> {
        let mut size = 0;
        size += self.message_header.encode(stream)?;
        size += self.protocol_version.encode(stream)?;
        size += self.receive_buffer_size.encode(stream)?;
        size += self.send_buffer_size.encode(stream)?;
        size += self.max_message_size.encode(stream)?;
        size += self.max_chunk_count.encode(stream)?;
        size += self.endpoint_url.encode(stream)?;
        Ok(size)
    }

    fn decode<S: Read>(stream: &mut S) -> EncodingResult<Self> {
        let message_header = MessageHeader::decode(stream)?;
        let protocol_version = UInt32::decode(stream)?;
        let receive_buffer_size = UInt32::decode(stream)?;
        let send_buffer_size = UInt32::decode(stream)?;
        let max_message_size = UInt32::decode(stream)?;
        let max_chunk_count = UInt32::decode(stream)?;
        let endpoint_url = UAString::decode(stream)?;
        Ok(HelloMessage {
            message_header: message_header,
            protocol_version: protocol_version,
            receive_buffer_size: receive_buffer_size,
            send_buffer_size: send_buffer_size,
            max_message_size: max_message_size,
            max_chunk_count: max_chunk_count,
            endpoint_url: endpoint_url,
        })
    }
}

impl HelloMessage {
    pub fn is_endpoint_url_valid(&self) -> bool {
        // TODO check server's endpoints
        if let Some(ref endpoint_url) = self.endpoint_url.value {
            if endpoint_url.len() > 4096 { false } else { true }
        } else {
            true
        }
    }

    pub fn is_valid_buffer_sizes(&self) -> bool {
        const MIN_BUFFER_SIZE: u32 = 8192; // TODO check specs - 8192 or 8196
        self.receive_buffer_size >= MIN_BUFFER_SIZE && self.send_buffer_size >= MIN_BUFFER_SIZE
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AcknowledgeMessage {
    pub message_header: MessageHeader,
    pub protocol_version: UInt32,
    pub receive_buffer_size: UInt32,
    pub send_buffer_size: UInt32,
    pub max_message_size: UInt32,
    pub max_chunk_count: UInt32,
}

impl BinaryEncoder<AcknowledgeMessage> for AcknowledgeMessage {
    fn byte_len(&self) -> usize {
        self.message_header.byte_len() + 20
    }

    fn encode<S: Write>(&self, stream: &mut S) -> EncodingResult<usize> {
        let mut size: usize = 0;
        size += self.message_header.encode(stream)?;
        size += self.protocol_version.encode(stream)?;
        size += self.receive_buffer_size.encode(stream)?;
        size += self.send_buffer_size.encode(stream)?;
        size += self.max_message_size.encode(stream)?;
        size += self.max_chunk_count.encode(stream)?;
        Ok(size)
    }

    fn decode<S: Read>(stream: &mut S) -> EncodingResult<Self> {
        let message_header = MessageHeader::decode(stream)?;
        let protocol_version = UInt32::decode(stream)?;
        let receive_buffer_size = UInt32::decode(stream)?;
        let send_buffer_size = UInt32::decode(stream)?;
        let max_message_size = UInt32::decode(stream)?;
        let max_chunk_count = UInt32::decode(stream)?;
        Ok(AcknowledgeMessage {
            message_header: message_header,
            protocol_version: protocol_version,
            receive_buffer_size: receive_buffer_size,
            send_buffer_size: send_buffer_size,
            max_message_size: max_message_size,
            max_chunk_count: max_chunk_count,
        })
    }
}

impl AcknowledgeMessage {}

#[derive(Debug, Clone, PartialEq)]
pub struct ErrorMessage {
    pub message_header: MessageHeader,
    pub error: UInt32,
    pub reason: UAString,
}

impl BinaryEncoder<ErrorMessage> for ErrorMessage {
    fn byte_len(&self) -> usize {
        self.message_header.byte_len() + 4 + self.reason.byte_len()
    }

    fn encode<S: Write>(&self, stream: &mut S) -> EncodingResult<usize> {
        let mut size: usize = 0;
        size += self.message_header.encode(stream)?;
        size += self.error.encode(stream)?;
        size += self.reason.encode(stream)?;
        Ok(size)
    }

    fn decode<S: Read>(stream: &mut S) -> EncodingResult<Self> {
        let message_header = MessageHeader::decode(stream)?;
        let error = UInt32::decode(stream)?;
        let reason = UAString::decode(stream)?;
        Ok(ErrorMessage {
            message_header: message_header,
            error: error,
            reason: reason,
        })
    }
}

impl ErrorMessage {
    pub fn from_status_code(status_code: &StatusCode) -> ErrorMessage {
        let mut error = ErrorMessage {
            message_header: MessageHeader::new(MessageType::Error),
            error: status_code.code,
            reason: UAString::from_str(status_code.description),
        };
        error.message_header.message_size = error.byte_len() as UInt32;
        error
    }
}