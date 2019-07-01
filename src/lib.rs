#![warn(unused_extern_crates)]

use std::io::{self, Read};

use protobuf::{Message, ProtobufResult};
use protobuf::stream::CodedInputStream;

pub mod error;
#[allow(dead_code, non_upper_case_globals, non_camel_case_types, unused_macros)]
pub mod machine;
pub mod protos;
pub mod symbolicate;
pub mod text;

pub use self::protos::crash_report::CrashReport;
pub use self::text::text_report;
pub use symbolicate::{Symbolicate, Location};

pub fn read_report<R: Read>(read: &mut R) -> ProtobufResult<CrashReport> {
    let mut header = [0u8; 8];
    read.read_exact(&mut header)?;
    let magic = &header[0..7];
    let version = header[7];
    if magic != b"plcrash" {
        return Err(io::Error::new(io::ErrorKind::InvalidInput,
            format!("Wrong magic {:?} != {:?}", magic, b"plcrash")).into());
    }
    if version != 1 {
        return Err(io::Error::new(io::ErrorKind::InvalidInput,
            format!("Wrong version = {}", version)).into());
    }
    let mut ret = CrashReport::new();
    let mut is = CodedInputStream::new(read);
    ret.merge_from(&mut is)?;
    Ok(ret)
}

/*
pub fn parse_report(bytes: &[u8]) -> ProtobufResult<CrashReport> {
    let mut ret = CrashReport::new();
    ret.merge_from_bytes(bytes)?;
    Ok(ret)
}
*/
