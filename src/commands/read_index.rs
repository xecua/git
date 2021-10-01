#![allow(unused)]
use anyhow::{bail, Error};
use git2::Object;
use std::convert::{TryFrom, TryInto};
use std::fs::File;
use std::io::Read;
use std::path::Path;

use log::debug;

use chrono::{Local, TimeZone};

use crate::errors::GeneralError;

#[derive(Clone, PartialEq, Eq)]
pub enum ObjectType {
    File,
    SymLink,
    GitLink, // submodule?
}

impl TryFrom<u8> for ObjectType {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0b1000 => Ok(Self::File),
            0b1010 => Ok(Self::SymLink),
            0b1110 => Ok(Self::GitLink),
            _ => bail!("Invalid object type value: {}", value),
        }
    }
}

impl std::fmt::Debug for ObjectType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::File => "File (1000)",
                Self::SymLink => "symlink (1010)",
                Self::GitLink => "gitlink (1110)",
            }
        )
    }
}

#[derive(Clone, PartialEq, Eq)]
pub enum Permission {
    Executable, // 0755
    Readable,   // 0644
    None,       // symlink/gitlink
}

impl TryFrom<u16> for Permission {
    type Error = Error;
    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            0o755 => Ok(Self::Executable),
            0o644 => Ok(Self::Readable),
            0 => Ok(Self::None),
            _ => bail!("Invalid permission type: {}", value),
        }
    }
}

impl std::fmt::Debug for Permission {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Executable => "Executable (755)",
                Self::Readable => "Readable (644)",
                Self::None => "symlink/gitlink (0)",
            }
        )
    }
}

#[derive(Debug)]
pub struct EntryExtend {
    skip_worktree_flag: bool,
    intent_to_add_flag: bool,
}

#[derive(Debug)]
pub struct Entry {
    c_time: u32,
    c_time_nano: u32,
    m_time: u32,
    m_time_nano: u32,
    dev: u32,
    ino: u32,
    object_type: ObjectType,
    permission: Permission,
    uid: u32,
    gid: u32,
    file_size: u32,
    object_name: String,
    assume_valid_flag: bool,
    staged_flag: (bool, bool),
    name_length: u16,
    extend: Option<EntryExtend>,
    path_name: String,
}

#[derive(Debug)]
pub struct Index {
    version: u32,
    entries: Vec<Entry>,
}

impl Index {
    pub fn new() -> Index {
        Index {
            version: 0,
            entries: Vec::new(),
        }
    }

    pub fn with_version(version: u32) -> Index {
        Index {
            version,
            entries: Vec::new(),
        }
    }

    pub fn set_version(&mut self, version: u32) {
        self.version = version;
    }

    pub fn push_entry(&mut self, entry: Entry) {
        self.entries.push(entry);
    }
}

pub fn get_index(repo_path: &Path) -> anyhow::Result<Index> {
    let mut index = File::open(repo_path.join("index"))?;
    let mut buf = Vec::new();

    index.read_to_end(&mut buf)?;

    if &buf[..4] != "DIRC".as_bytes() {
        bail!(GeneralError::InvalidFormat(
            "Index file header was invalid.".to_string(),
        ));
    }

    let version = u32::from_be_bytes(buf[4..8].try_into()?);
    if version != 2 && version != 3 && version != 4 {
        bail!(GeneralError::InvalidFormat(
            "Index file version was invalid.".to_string(),
        ));
    }

    if version == 4 {
        unimplemented!("This feature is not yet implemented for index file version 4...");
    }

    let mut index = Index::with_version(version);

    let num_entries = u32::from_be_bytes(buf[8..12].try_into()?);
    debug!("version: {}, entries: {}", version, num_entries);
    let mut offset = 12;
    for _ in 0..num_entries {
        // entry
        let mut entry_size = 0;
        let c_time = u32::from_be_bytes(buf[offset..offset + 4].try_into()?);
        offset += 4;
        entry_size += 4;
        let c_time_nano = u32::from_be_bytes(buf[offset..offset + 4].try_into()?);
        log::debug!(
            "metadata changed: {}",
            Local.timestamp(c_time.into(), c_time_nano).to_string()
        );
        offset += 4;
        entry_size += 4;
        let m_time = u32::from_be_bytes(buf[offset..offset + 4].try_into()?);
        offset += 4;
        entry_size += 4;
        let m_time_nano = u32::from_be_bytes(buf[offset..offset + 4].try_into()?);
        offset += 4;
        entry_size += 4;
        log::debug!(
            "content modified: {}",
            Local.timestamp(m_time.into(), m_time_nano).to_string()
        );
        let dev = u32::from_be_bytes(buf[offset..offset + 4].try_into()?);
        offset += 4;
        entry_size += 4;
        let ino = u32::from_be_bytes(buf[offset..offset + 4].try_into()?);
        offset += 4;
        entry_size += 4;

        offset += 2;
        entry_size += 2;
        let object_type: ObjectType = ((0b11110000 & buf[offset]) >> 4).try_into()?;
        let permission: Permission =
            ((u16::from(0b1 & buf[offset]) << 8) + u16::from(buf[offset + 1])).try_into()?;
        offset += 2;
        entry_size += 2;
        log::debug!(
            "dev: {} / ino: {} / object_type: {:?} / permission: {:?}",
            dev,
            ino,
            object_type,
            permission
        );

        let uid = u32::from_be_bytes(buf[offset..offset + 4].try_into()?);
        offset += 4;
        entry_size += 4;
        let gid = u32::from_be_bytes(buf[offset..offset + 4].try_into()?);
        offset += 4;
        entry_size += 4;
        log::debug!("uid: {} / gid: {}", uid, gid);
        let file_size = u32::from_be_bytes(buf[offset..offset + 4].try_into()?);
        offset += 4;
        entry_size += 4;
        log::debug!("file size (32bit truncated): {}", file_size);
        let object_name = buf[offset..offset + 20]
            .iter()
            .map(|n| format!("{:02X}", n))
            .collect::<String>();
        offset += 20;
        entry_size += 20;
        log::debug!("object name: {}", object_name);

        let assume_valid_flag = (0b10000000 & buf[offset]) >> 7 == 1;
        let extend_flag = (0b01000000 & buf[offset]) >> 6 == 1;
        if version == 2 && extend_flag {
            bail!(GeneralError::InvalidFormat(
                "Extend flag cannot be used in index file version 2.".to_string(),
            ));
        }
        let staged_flag = (
            (0b100000 & buf[offset]) >> 5 == 1,
            (0b10000 & buf[offset]) >> 4 == 1,
        );
        log::debug!(
            "assume-valid: {} / extend: {} / staged: {}{}",
            assume_valid_flag,
            extend_flag,
            staged_flag.0,
            staged_flag.1
        );

        let name_length = (u16::from(0b1111 & buf[offset]) << 8) + u16::from(buf[offset + 1]);
        log::debug!("name length: {}", name_length);
        offset += 2;
        entry_size += 2;

        let (skip_worktree_flag, intent_to_add_flag) = if extend_flag {
            let skip_worktree_flag = (0b1000000 & buf[offset]) >> 6 == 1;
            let intent_to_add_flag = (0b100000 & buf[offset]) >> 5 == 1;
            if 0b11111 & buf[offset] != 0 || buf[offset + 1] != 0 {
                bail!(GeneralError::InvalidFormat(
                    "The field which must be zero was not zero.".to_string(),
                ));
            }
            offset += 2;
            entry_size += 2;
            (Some(skip_worktree_flag), Some(intent_to_add_flag))
        } else {
            (None, None)
        };
        log::debug!(
            "skip-worktree: {:?} / intent-to-add: {:?}",
            skip_worktree_flag,
            intent_to_add_flag
        );

        // version 4だとまるっきし違うらしい
        let path_name = {
            // null文字まで読み進める?
            let mut name = Vec::new();
            while buf[offset] != 0u8 {
                name.push(buf[offset]);
                offset += 1;
            }
            offset += 1;
            log::debug!("{:?}", name);
            String::from_utf8(name)?
        };
        entry_size += path_name.len() + 1;

        log::debug!("{}", path_name);
        offset += 8 - (entry_size % 8);

        let entry = Entry {
            c_time,
            c_time_nano,
            m_time,
            m_time_nano,
            dev,
            ino,
            object_type,
            permission,
            uid,
            gid,
            file_size,
            object_name,
            assume_valid_flag,
            staged_flag,
            name_length,
            path_name,
            extend: match (skip_worktree_flag, intent_to_add_flag) {
                (Some(skip_worktree_flag), Some(intent_to_add_flag)) => Some(EntryExtend {
                    skip_worktree_flag,
                    intent_to_add_flag,
                }),
                _ => None,
            },
        };
        index.push_entry(entry);
    }

    Ok(index)
}

pub fn read_index(repo_path: &Path) -> anyhow::Result<()> {
    println!("{:#?}", get_index(repo_path)?);
    Ok(())
}
