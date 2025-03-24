#![allow(dead_code)]

/// Internal module for handling circular dependencies among messages.
mod circular;
/// Module containing definitions for protocol buffer fields.
pub mod field;
/// Module containing definitions for protocol buffer map fields.
pub mod map;
/// Module containing definitions for protocol buffer messages.
pub mod message;
/// Module containing definitions for protocol buffer oneof groups.
pub mod one_of;
/// Module containing definitions for protocol buffer packages.
pub mod package;
/// Module containing definitions for protocol buffer enumerations.
pub mod proto_enum;
/// Module containing definitions for protocol buffer schemas.
pub mod schema;
/// Module containing definitions for protocol buffer services.
pub mod service;
/// Module containing writer utilities for protocol buffers.
pub mod writer;

use crate::proto::message::ProtoMessage;
use crate::proto::proto_enum::ProtoEnum;
use crate::proto::service::ProtoService;

/// Represents a protocol buffer type, which can be an enumeration, a message, or a service.
#[derive(PartialEq)]
pub enum ProtoType {
    /// A protocol enumeration.
    Enum(ProtoEnum),
    /// A protocol message.
    Message(ProtoMessage),
    /// A protocol service.
    Service(ProtoService),
}
