// OPCUA for Rust
// SPDX-License-Identifier: MPL-2.0
// Copyright (C) 2017-2020 Adam Lock

// This file was autogenerated from Opc.Ua.Types.bsd.xml by tools/schema/gen_types.js
// DO NOT EDIT THIS FILE
#![rustfmt::skip]

use std::io::{Read, Write};

#[allow(unused_imports)]
use crate::{
    encoding::*,
    basic_types::*,
    service_types::impls::MessageInfo,
    node_ids::ObjectId,
    string::UAString,
};

#[derive(Debug, Clone, PartialEq)]
pub struct SessionlessInvokeResponseType {
    pub namespace_uris: Option<Vec<UAString>>,
    pub server_uris: Option<Vec<UAString>>,
    pub service_id: u32,
}

impl MessageInfo for SessionlessInvokeResponseType {
    fn object_id(&self) -> ObjectId {
        ObjectId::SessionlessInvokeResponseType_Encoding_DefaultBinary
    }
}

impl BinaryEncoder<SessionlessInvokeResponseType> for SessionlessInvokeResponseType {
    fn byte_len(&self) -> usize {
        let mut size = 0;
        size += byte_len_array(&self.namespace_uris);
        size += byte_len_array(&self.server_uris);
        size += self.service_id.byte_len();
        size
    }

    #[allow(unused_variables)]
    fn encode<S: Write>(&self, stream: &mut S) -> EncodingResult<usize> {
        let mut size = 0;
        size += write_array(stream, &self.namespace_uris)?;
        size += write_array(stream, &self.server_uris)?;
        size += self.service_id.encode(stream)?;
        Ok(size)
    }

    #[allow(unused_variables)]
    fn decode<S: Read>(stream: &mut S, decoding_limits: &DecodingLimits) -> EncodingResult<Self> {
        let namespace_uris: Option<Vec<UAString>> = read_array(stream, decoding_limits)?;
        let server_uris: Option<Vec<UAString>> = read_array(stream, decoding_limits)?;
        let service_id = u32::decode(stream, decoding_limits)?;
        Ok(SessionlessInvokeResponseType {
            namespace_uris,
            server_uris,
            service_id,
        })
    }
}
