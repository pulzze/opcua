use std::io::{Read, Write, Result};

use super::helpers::*;
use super::status_codes::*;
use super::node_id::*;

// OPC UA Part 6 - Mappings 1.03 Specification

/// OPC UA Binary Encoding interface. Anything that encodes to binary must implement this.
pub trait BinaryEncoder<T> {
    /// Returns the byte length of the structure. This calculation should be exact and as efficient
    /// as possible.
    fn byte_len(&self) -> usize;
    /// Encodes the instance to the write stream.
    fn encode(&self, _: &mut Write) -> Result<usize>;
    /// Decodes an instance from the read stream.
    fn decode(_: &mut Read) -> Result<T>;
}

// These are standard UA types

/// A two-state logical value (true or false).
/// Data type ID 1
pub type Boolean = bool;

impl BinaryEncoder<Boolean> for Boolean {
    fn byte_len(&self) -> usize {
        1
    }

    fn encode(&self, stream: &mut Write) -> Result<usize> {
        // 0, or 1 for true or false, single byte
        write_u8(stream, if *self { 1 } else { 0 })
    }

    fn decode(stream: &mut Read) -> Result<Boolean> {
        let value = if read_u8(stream)? == 1 { true } else { false };
        Ok(value)
    }
}

/// An integer value between −128 and 127.
/// Data type ID 2
pub type SByte = i8;

impl BinaryEncoder<SByte> for SByte {
    fn byte_len(&self) -> usize {
        1
    }

    fn encode(&self, stream: &mut Write) -> Result<usize> {
        write_u8(stream, *self as u8)
    }

    fn decode(stream: &mut Read) -> Result<SByte> {
        Ok(read_u8(stream)? as i8)
    }
}

/// An integer value between 0 and 255.
/// Data type ID 3
pub type Byte = u8;

impl BinaryEncoder<Byte> for Byte {
    fn byte_len(&self) -> usize {
        1
    }

    fn encode(&self, stream: &mut Write) -> Result<usize> {
        write_u8(stream, *self)
    }

    fn decode(stream: &mut Read) -> Result<Byte> {
        Ok(read_u8(stream)?)
    }
}

/// An integer value between −32 768 and 32 767.
/// Data type ID 4
pub type Int16 = i16;

impl BinaryEncoder<Int16> for Int16 {
    fn byte_len(&self) -> usize {
        2
    }

    fn encode(&self, stream: &mut Write) -> Result<usize> {
        write_i16(stream, *self)
    }

    fn decode(stream: &mut Read) -> Result<Int16> {
        read_i16(stream)
    }
}

/// An integer value between 0 and 65 535.
/// Data type ID 5
pub type UInt16 = u16;

impl BinaryEncoder<UInt16> for UInt16 {
    fn byte_len(&self) -> usize {
        2
    }

    fn encode(&self, stream: &mut Write) -> Result<usize> {
        write_u16(stream, *self)
    }

    fn decode(stream: &mut Read) -> Result<UInt16> {
        read_u16(stream)
    }
}

/// An integer value between −2 147 483 648 and 2 147 483 647.
/// Data type ID 6
pub type Int32 = i32;

impl BinaryEncoder<Int32> for Int32 {
    fn byte_len(&self) -> usize {
        4
    }

    fn encode(&self, stream: &mut Write) -> Result<usize> {
        write_i32(stream, *self)
    }

    fn decode(stream: &mut Read) -> Result<Int32> {
        read_i32(stream)
    }
}

/// An integer value between 0 and 4 294 967 295.
/// Data type ID 7
pub type UInt32 = u32;

impl BinaryEncoder<UInt32> for UInt32 {
    fn byte_len(&self) -> usize {
        4
    }

    fn encode(&self, stream: &mut Write) -> Result<usize> {
        write_u32(stream, *self)
    }

    fn decode(stream: &mut Read) -> Result<UInt32> {
        read_u32(stream)
    }
}

/// An integer value between −9 223 372 036 854 775 808 and 9 223 372 036 854 775 807
/// Data type ID 8
pub type Int64 = i64;

impl BinaryEncoder<Int64> for Int64 {
    fn byte_len(&self) -> usize {
        8
    }

    fn encode(&self, stream: &mut Write) -> Result<usize> {
        write_i64(stream, *self)
    }

    fn decode(stream: &mut Read) -> Result<Int64> {
        read_i64(stream)
    }
}

/// An integer value between 0 and 18 446 744 073 709 551 615.
/// Data type ID 9
pub type UInt64 = u64;

impl BinaryEncoder<UInt64> for UInt64 {
    fn byte_len(&self) -> usize {
        8
    }

    fn encode(&self, stream: &mut Write) -> Result<usize> {
        write_u64(stream, *self)
    }

    fn decode(stream: &mut Read) -> Result<UInt64> {
        read_u64(stream)
    }
}

/// An IEEE single precision (32 bit) floating point value.
/// Data type ID 10
pub type Float = f32;

impl BinaryEncoder<Float> for Float {
    fn byte_len(&self) -> usize {
        4
    }

    fn encode(&self, stream: &mut Write) -> Result<usize> {
        write_f32(stream, *self)
    }

    fn decode(stream: &mut Read) -> Result<Float> {
        read_f32(stream)
    }
}

/// An IEEE double precision (64 bit) floating point value.
/// Data type ID 11
pub type Double = f64;

impl BinaryEncoder<Double> for Double {
    fn byte_len(&self) -> usize {
        8
    }

    fn encode(&self, stream: &mut Write) -> Result<usize> {
        write_f64(stream, *self)
    }

    fn decode(stream: &mut Read) -> Result<Double> {
        read_f64(stream)
    }
}

/// A sequence of Unicode characters.
/// A UA string can hold a null value, so the string value is optional.
/// When there is no string, the value is treated as null
/// Data type ID 12
#[derive(PartialEq, Debug, Clone)]
pub struct UAString {
    pub value: Option<String>,
}

impl BinaryEncoder<UAString> for UAString {
    fn byte_len(&self) -> usize {
        // Length plus the actual length of bytes (if not null)
        4 + if self.value.is_none() { 0 } else { self.value.as_ref().unwrap().len() }
    }

    fn encode(&self, stream: &mut Write) -> Result<usize> {
        // Strings are uncoded as UTF8 chars preceded by an Int32 length. A -1 indicates a null string
        if self.value.is_none() {
            write_i32(stream, -1)
        } else {
            let value = self.value.clone().unwrap();
            let mut size: usize = 0;
            size += write_i32(stream, value.len() as i32)?;
            let buf = value.as_bytes();
            size += stream.write(&buf)?;
            Ok(size)
        }
    }

    fn decode(stream: &mut Read) -> Result<UAString> {
        let buf_len = read_i32(stream)?;
        // Null string?
        if buf_len == -1 {
            return Ok(UAString { value: None });
        }
        // Create the actual UTF8 string
        let mut string_buf: Vec<u8> = Vec::with_capacity(buf_len as usize);
        string_buf.resize(buf_len as usize, 0u8);
        stream.read_exact(&mut string_buf)?;
        Ok(UAString { value: Some(String::from_utf8(string_buf).unwrap()) })
    }
}

impl UAString {
    /// Create a string from a string slice
    pub fn from_str(value: &str) -> UAString {
        UAString { value: Some(value.to_string()) }
    }

    /// Returns the length of the string or -1 for null
    pub fn len(&self) -> isize {
        if self.value.is_none() { -1 } else { self.value.as_ref().unwrap().len() as isize }
    }

    /// Create a null string (not the same as an empty string)
    pub fn null_string() -> UAString {
        UAString { value: None }
    }

    /// Test if the string is null
    pub fn is_null(&self) -> bool {
        self.value.is_none()
    }
}

// Data type ID 13 - UADateTime is in date_time.rs

/// A 16 byte value that can be used as a globally unique identifier.
/// Data type ID 14
use std::fmt;

#[derive(PartialEq, Clone)]
pub struct Guid {
    pub data1: UInt32,
    pub data2: UInt16,
    pub data3: UInt16,
    pub data4: [Byte; 8],
}

impl fmt::Debug for Guid {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:08X}-{:04X}-{:04X}-{:02X}{:02X}-{:02X}{:02X}{:02X}{:02X}{:02X}{:02X}",
               self.data1, self.data2, self.data3, self.data4[0], self.data4[1], self.data4[2], self.data4[3], self.data4[4], self.data4[5], self.data4[6], self.data4[7])
    }
}

impl BinaryEncoder<Guid> for Guid {
    fn byte_len(&self) -> usize {
        16
    }

    fn encode(&self, stream: &mut Write) -> Result<usize> {
        let mut size: usize = 0;
        size += write_u32(stream, self.data1)?;
        size += write_u16(stream, self.data2)?;
        size += write_u16(stream, self.data3)?;
        size += stream.write(&self.data4)?;
        assert_eq!(size, 16);
        Ok(size)
    }

    fn decode(stream: &mut Read) -> Result<Guid> {
        let data1 = read_u32(stream)?;
        let data2 = read_u16(stream)?;
        let data3 = read_u16(stream)?;
        let mut data4: [u8; 8] = [0; 8];
        stream.read_exact(&mut data4)?;
        Ok(Guid { data1: data1, data2: data2, data3: data3, data4: data4, })
    }
}

/// A sequence of octets.
/// Data type ID 15
pub type ByteString = UAString;

/// An XML element.
/// Data type ID 16
pub type XmlElement = UAString;

// NodeId and ExtendedNodeId are in node_id.rs

/// A numeric identifier for a error or condition that is associated with a value or an operation.
/// Data type ID 19

/// A name qualified by a namespace.
/// Data type ID 20
#[derive(PartialEq, Debug, Clone)]
pub struct QualifiedName {
    /// The namespace index.
    pub namespace_index: UInt16,
    /// The name.
    pub name: UAString,
}

impl BinaryEncoder<QualifiedName> for QualifiedName {
    fn byte_len(&self) -> usize {
        let mut size: usize = 0;
        size += self.namespace_index.byte_len();
        size += self.name.byte_len();
        size
    }

    fn encode(&self, stream: &mut Write) -> Result<usize> {
        let mut size: usize = 0;
        size += self.namespace_index.encode(stream)?;
        size += self.name.encode(stream)?;
        Ok(size)
    }

    fn decode(stream: &mut Read) -> Result<QualifiedName> {
        let namespace_index = read_u16(stream)?;
        let name = UAString::decode(stream)?;
        Ok(QualifiedName {
            namespace_index: namespace_index,
            name: name,
        })
    }
}

/// Human readable text with an optional locale identifier
/// Data type ID 21
#[derive(PartialEq, Debug, Clone)]
pub struct LocalizedText {
    /// A bit mask that indicates which fields are present in the stream.
    /// The mask has the following bits:
    /// 0x01    Locale
    /// 0x02    Text
    pub encoding_mask: Byte,
    /// The locale. Omitted is null or empty
    pub locale: Option<UAString>,
    /// The text in the specified locale. Omitted is null or empty.
    pub text: Option<UAString>,
}

impl BinaryEncoder<LocalizedText> for LocalizedText {
    fn byte_len(&self) -> usize {
        unimplemented!();
    }

    fn encode(&self, _: &mut Write) -> Result<usize> {
        // This impl should be overridden
        unimplemented!()
    }

    fn decode(_: &mut Read) -> Result<LocalizedText> {
        // This impl should be overridden
        unimplemented!()
    }
}

#[derive(PartialEq, Debug, Clone)]
pub enum ExtensionObjectEncoding {
    None,
    ByteString(ByteString),
    XmlElement(XmlElement),
}

/// A structure that contains an application specific data type that may not be recognized by the receiver.
/// Data type ID 22
#[derive(PartialEq, Debug, Clone)]
pub struct ExtensionObject {
    pub node_id: NodeId,
    pub body: ExtensionObjectEncoding,
}

impl BinaryEncoder<ExtensionObject> for ExtensionObject {
    fn byte_len(&self) -> usize {
        let mut size = self.node_id.byte_len();
        size += match self.body {
            ExtensionObjectEncoding::None => 0,
            ExtensionObjectEncoding::ByteString(ref value) => {
                // Encoding mask + data
                1 + value.byte_len()
            },
            ExtensionObjectEncoding::XmlElement(ref value) => {
                // Encoding mask + data
                1 + value.byte_len()
            },
        };
        size
    }

    fn encode(&self, stream: &mut Write) -> Result<usize> {
        let mut size = 0;
        size += self.node_id.encode(stream)?;

        match self.body {
            ExtensionObjectEncoding::None => {},
            ExtensionObjectEncoding::ByteString(ref value) => {
                // Encoding mask + data
                size += write_u8(stream, 0x1)?;
                size += value.encode(stream)?;
            },
            ExtensionObjectEncoding::XmlElement(ref value) => {
                // Encoding mask + data
                size += write_u8(stream, 0x2)?;
                size += value.encode(stream)?;
            },
        }
        Ok(size)
    }

    fn decode(stream: &mut Read) -> Result<ExtensionObject> {
        let node_id = NodeId::decode(stream)?;
        let encoding_type = Byte::decode(stream)?;
        let body = match encoding_type {
            0x0 => {
                ExtensionObjectEncoding::None
            },
            0x1 => {
                let value = ByteString::decode(stream);
                if value.is_err() {
                    return Err(value.unwrap_err());
                }
                ExtensionObjectEncoding::ByteString(value.unwrap())
            },
            0x2 => {
                let value = XmlElement::decode(stream);
                if value.is_err() {
                    return Err(value.unwrap_err());
                }
                ExtensionObjectEncoding::XmlElement(value.unwrap())
            },
            _ => {
                error!("Invalid encoding type {} in stream", encoding_type);
                // TODO Err()
                ExtensionObjectEncoding::None
            }
        };
        Ok(ExtensionObject {
            node_id: node_id,
            body: body,
        })
    }
}

impl ExtensionObject {
    pub fn null() -> ExtensionObject {
        ExtensionObject {
            node_id: NodeId::null(),
            body: ExtensionObjectEncoding::None,
        }
    }
}

// Data type ID 23 is in data_value.rs

// Data type ID 24 is in variant.rs

#[allow(non_snake_case)]
mod DiagnosticInfoMask {
    pub const HAS_SYMBOLIC_ID: u8 = 0x01;
    pub const HAS_NAMESPACE: u8 = 0x02;
    pub const HAS_LOCALIZED_TEXT: u8 = 0x04;
    pub const HAS_LOCALE: u8 = 0x08;
    pub const HAS_ADDITIONAL_INFO: u8 = 0x10;
    pub const HAS_INNER_STATUS_CODE: u8 = 0x20;
    pub const HAS_INNER_DIAGNOSTIC_INFO: u8 = 0x40;
}

/// Data type ID 25
#[derive(PartialEq, Debug, Clone)]
pub struct DiagnosticInfo {
    /// A symbolic name for the status code.
    pub symbolic_id: Option<Int32>,

    /// A namespace that qualifies the symbolic id.
    pub namespace_uri: Option<Int32>,

    /// The locale used for the localized text.
    pub locale: Option<Int32>,

    /// A human readable summary of the status code.
    pub localized_text: Option<Int32>,

    /// Detailed application specific diagnostic information.
    pub additional_info: Option<UAString>,

    /// A status code provided by an underlying system.
    pub inner_status_code: Option<StatusCode>,

    /// Diagnostic info associated with the inner status code.
    pub inner_diagnostic_info: Option<Box<DiagnosticInfo>>,
}

impl BinaryEncoder<DiagnosticInfo> for DiagnosticInfo {
    fn byte_len(&self) -> usize {
        let mut size: usize = 0;
        size += 1; // self.encoding_mask())
        if let Some(ref symbolic_id) = self.symbolic_id {
            // Write symbolic id
            size += symbolic_id.byte_len();
        }
        if let Some(ref namespace_uri) = self.namespace_uri {
            // Write namespace
            size += namespace_uri.byte_len()
        }
        if let Some(ref localized_text) = self.localized_text {
            // Write localized text
            size += localized_text.byte_len()
        }
        if let Some(ref locale) = self.locale {
            // Write locale
            size += locale.byte_len()
        }
        if let Some(ref additional_info) = self.additional_info.clone() {
            // Write Additional info
            size += additional_info.byte_len()
        }
        if let Some(ref inner_status_code) = self.inner_status_code {
            // Write inner status code
            size += inner_status_code.byte_len()
        }
        if let Some(ref inner_diagnostic_info) = self.inner_diagnostic_info {
            // Write inner diagnostic info
            size += inner_diagnostic_info.byte_len()
        }
        size
    }

    fn encode(&self, stream: &mut Write) -> Result<usize> {
        let mut size: usize = 0;
        size += write_u8(stream, self.encoding_mask())?;
        if let Some(ref symbolic_id) = self.symbolic_id {
            // Write symbolic id
            size += write_i32(stream, *symbolic_id)?;
        }
        if let Some(ref namespace_uri) = self.namespace_uri {
            // Write namespace
            size += write_i32(stream, *namespace_uri)?;
        }
        if let Some(ref localized_text) = self.localized_text {
            // Write localized text
            size += write_i32(stream, *localized_text)?;
        }
        if let Some(ref locale) = self.locale {
            // Write locale
            size += write_i32(stream, *locale)?;
        }
        if let Some(ref additional_info) = self.additional_info.clone() {
            // Write Additional info
            size += additional_info.encode(stream)?;
        }
        if let Some(ref inner_status_code) = self.inner_status_code {
            // Write inner status code
            size += inner_status_code.clone().encode(stream)?;
        }
        if let Some(ref inner_diagnostic_info) = self.inner_diagnostic_info {
            // Write inner diagnostic info
            size += inner_diagnostic_info.clone().encode(stream)?;
        }
        Ok(size)
    }

    fn decode(stream: &mut Read) -> Result<DiagnosticInfo> {
        let encoding_mask = read_u8(stream)?;
        let mut diagnostic_info = DiagnosticInfo::new();
        if encoding_mask & DiagnosticInfoMask::HAS_SYMBOLIC_ID != 0 {
            // Read symbolic id
            diagnostic_info.symbolic_id = Some(read_i32(stream)?);
        }
        if encoding_mask & DiagnosticInfoMask::HAS_NAMESPACE != 0 {
            // Read namespace
            diagnostic_info.namespace_uri = Some(read_i32(stream)?);
        }
        if encoding_mask & DiagnosticInfoMask::HAS_LOCALIZED_TEXT != 0 {
            // Read localized text
            diagnostic_info.localized_text = Some(read_i32(stream)?);
        }
        if encoding_mask & DiagnosticInfoMask::HAS_LOCALE != 0 {
            // Read locale
            diagnostic_info.locale = Some(read_i32(stream)?);
        }
        if encoding_mask & DiagnosticInfoMask::HAS_ADDITIONAL_INFO != 0 {
            // Read Additional info
            diagnostic_info.additional_info = Some(UAString::decode(stream)?);
        }
        if encoding_mask & DiagnosticInfoMask::HAS_INNER_STATUS_CODE != 0 {
            // Read inner status code
            diagnostic_info.inner_status_code = Some(StatusCode::decode(stream)?);
        }
        if encoding_mask & DiagnosticInfoMask::HAS_INNER_DIAGNOSTIC_INFO != 0 {
            // Read inner diagnostic info
            diagnostic_info.inner_diagnostic_info = Some(Box::new(DiagnosticInfo::decode(stream)?));
        }
        Ok(diagnostic_info)
    }
}

impl DiagnosticInfo {
    pub fn new() -> DiagnosticInfo {
        DiagnosticInfo {
            symbolic_id: None,
            namespace_uri: None,
            locale: None,
            localized_text: None,
            additional_info: None,
            inner_status_code: None,
            inner_diagnostic_info: None,
        }
    }

    pub fn encoding_mask(&self) -> u8 {
        let mut encoding_mask: u8 = 0;
        if self.symbolic_id.is_some() {
            encoding_mask |= DiagnosticInfoMask::HAS_SYMBOLIC_ID;
        }
        if self.namespace_uri.is_some() {
            encoding_mask |= DiagnosticInfoMask::HAS_NAMESPACE;
        }
        if self.localized_text.is_some() {
            encoding_mask |= DiagnosticInfoMask::HAS_LOCALIZED_TEXT;
        }
        if self.locale.is_some() {
            encoding_mask |= DiagnosticInfoMask::HAS_LOCALE;
        }
        if self.additional_info.is_some() {
            encoding_mask |= DiagnosticInfoMask::HAS_ADDITIONAL_INFO;
        }
        if self.inner_status_code.is_some() {
            encoding_mask |= DiagnosticInfoMask::HAS_INNER_STATUS_CODE;
        }
        if self.inner_diagnostic_info.is_some() {
            encoding_mask |= DiagnosticInfoMask::HAS_INNER_DIAGNOSTIC_INFO;
        }
        encoding_mask
    }
}