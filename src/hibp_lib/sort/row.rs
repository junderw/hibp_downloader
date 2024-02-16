use std::{
    io::{Read, Write},
    str::FromStr,
};

use anyhow::Context;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use extsort::Sortable;

#[derive(Debug)]
pub struct MyStruct {
    pub count: u32,
    pub hash: String,
}

#[allow(clippy::non_canonical_partial_ord_impl)]
impl PartialOrd for MyStruct {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        // reverse order (highest to lowest)
        other.count.partial_cmp(&self.count)
    }
}

impl Ord for MyStruct {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // reverse order (highest to lowest)
        other.count.cmp(&self.count)
    }
}

impl PartialEq for MyStruct {
    fn eq(&self, other: &Self) -> bool {
        self.count == other.count
    }
}

impl Eq for MyStruct {}

impl FromStr for MyStruct {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (hash, count) = s.split_once(':').context("Line missing colon")?;
        let count = count.parse::<u32>()?;
        Ok(Self {
            count,
            hash: hash.to_string(),
        })
    }
}

impl Sortable for MyStruct {
    fn encode<W: Write>(&self, write: &mut W) {
        // in case of NTLM we need to write the length
        write
            .write_u8(self.hash.len() as u8)
            .expect("Write failure");
        write
            .write_all(self.hash.as_bytes())
            .expect("Write failure");
        write
            .write_u32::<LittleEndian>(self.count)
            .expect("Write failure");
    }

    fn decode<R: Read>(read: &mut R) -> Option<MyStruct> {
        let mut result = [0_u8; 40];
        // read the hash length
        read.read_exact(&mut result[..1]).ok()?;
        let hash_len = result[0] as usize;

        // read the hash
        read.read_exact(&mut result[..hash_len]).ok()?;

        let count = read.read_u32::<LittleEndian>().ok()?;
        Some(MyStruct {
            count,
            hash: String::from_utf8(result[..hash_len].to_vec()).ok()?,
        })
    }
}
