use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use addr2line::{Context, FrameIter};
use failure::{Fail, ResultExt};
use gimli::{EndianRcSlice, RunTimeEndian};
use goblin::{peek_bytes, Hint};
use goblin::mach;
use goblin::mach::load_command::CommandVariant;
use serde::Deserialize;
//use lru::LruCache;
use uuid::Uuid;

use crate::error::{Error, ErrorKind};
//use version_info::VersionInfo;

/*
pub struct Cache {
    versions: HashMap<String, VersionInfo>,
    lru: LruCache<String, Symbolicate>,
    disk_cache: PathBuf,
}

impl Cache {
    pub fn new(versions: HashMap<String, VersionInfo>, disk_cache: PathBuf) -> Cache {
        Cache {
            versions: versions,
            lru: LruCache::new(10),
            disk_cache: disk_cache,
        }
    }

    pub fn get(&mut self, version: &String) -> Result<Option<&Symbolicate>, Error> {
        if self.lru.contains(version) {
            return Ok(self.lru.get(version));
        }

        if let Some(info) = self.versions.get(version) {
            if let &Some(ref _dsym) = &info.dsym {
                let cache_path = self.disk_cache.join(format!("{}.zip", version));
                /*if !cache_path.exists() {
                    let file = File::create(&cache_path)?;
                }*/
                let symbolicate = Symbolicate::new(&cache_path)?;
                self.lru.put(version.clone(), symbolicate);
            } else {
                return Ok(None);
            }
        } else {
            return Ok(None);
        }

        Ok(self.lru.get(version))
    }
}
*/

#[derive(Clone, Debug, Deserialize)]
pub struct DSYMInfo {
    #[serde(rename = "CFBundleIdentifier")]
    pub identifier: String,
    #[serde(rename = "CFBundleVersion")]
    pub version: String,
    #[serde(rename = "CFBundleShortVersionString")]
    pub short_version: Option<String>,
}

impl fmt::Display for DSYMInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(short) = self.short_version.as_ref() {
            write!(f, "{} ({} - {})", self.identifier, short, self.version)
        } else {
            write!(f, "{} ({})", self.identifier, self.version)
        }
    }
}

pub struct Symbolicate {
    files: HashMap<Uuid, Lookup>,
}

impl Symbolicate {
    pub fn new<P:AsRef<Path>>(path: P) -> Result<Symbolicate, Error> {
        let kind = ErrorKind::Zip(path.as_ref().to_path_buf());
        let all = std::fs::read(&path).context(kind.clone())?;
        let reader = std::io::Cursor::new(all);
        let mut zip = zip::ZipArchive::new(reader).context(kind.clone())?;
        let mut dsyms = Vec::new();
        for i in 0..zip.len()
        {
            let file = zip.by_index(i).context(kind.clone())?;
            //println!("Filename: {}", file.name());
            if file.name().ends_with(".dSYM/") {
                dsyms.push(file.sanitized_name());
            }
        }
        let mut files = HashMap::new();
        for dsym in dsyms.into_iter() {
            let kind = ErrorKind::DSYM(path.as_ref().to_path_buf(), dsym.clone());

            let base_name = Path::new(dsym.file_stem().ok_or(kind.clone())?)
                .file_stem().ok_or(kind.clone())?;
            let info_name = dsym.join("Contents/Info.plist");
            let info : DSYMInfo = {
                let mut info_file = zip.by_name(info_name.to_str().unwrap()).context(kind.clone())?;
                let mut info_bytes = Vec::new();
                info_file.read_to_end(&mut info_bytes).context(kind.clone())?;
                let info_cursor = std::io::Cursor::new(info_bytes);
                plist::from_reader(info_cursor).context(kind.clone())?
            };
            eprintln!("Info {:?} {:?}", base_name, info);

            let mut dwarf_bytes = Vec::new();
            {
                let dwarf_name = dsym.join("Contents/Resources/DWARF").join(base_name);
                let mut dwarf_file = zip.by_name(dwarf_name.to_str().unwrap()).context(kind.clone())?;
                dwarf_file.read_to_end(&mut dwarf_bytes).context(kind.clone())?;

                let mut bytes = [0u8; 16];
                bytes.clone_from_slice(&dwarf_bytes[..16]);
                match peek_bytes(&bytes).context(kind.clone())? {
                    Hint::MachFat(count) => {
                        let multi = mach::MultiArch::new(&dwarf_bytes).context(kind.clone())?;
                        for idx in 0..count {
                            let macho = multi.get(idx).context(kind.clone())?;
                            let lookup = Lookup::load(dsym.clone(), info.clone(), &macho).context(kind.clone())?;
                            if let Some(uuid) = lookup.uuid {
                                files.insert(uuid, lookup);
                            }
                        }
                    },
                    Hint::Mach(_) => {
                        let macho = mach::MachO::parse(&dwarf_bytes, 0).context(kind.clone())?;
                        let lookup = Lookup::load(dsym.clone(), info.clone(), &macho).context(kind.clone())?;
                        if let Some(uuid) = lookup.uuid {
                            files.insert(uuid, lookup);
                        }
                    },
                    _ => {
                        return Err(kind)?;
                    }
                }
            }
        }

        Ok(Symbolicate {
            files: files,
        })
    }

    pub fn get(&self, uuid: &Uuid) -> Option<&Lookup> {
        self.files.get(uuid)
    }

    pub fn lookup(&self, uuid: &Uuid, probe: u64) -> Result<Option<Location>, Error> {
        if let Some(lookup) = self.files.get(uuid) {
            lookup.find_location(probe)
        } else {
            Ok(None)
        }
    }

    pub fn frames(&self, uuid: &Uuid, probe: u64) -> Result<Frames, Error> {
        if let Some(lookup) = self.files.get(uuid) {
            lookup.find_frames(probe)
        } else {
            Ok(Frames::Empty)
        }
    }
}

pub enum Frames<'ctx> {
    Actual(FrameIter<'ctx, EndianRcSlice<RunTimeEndian>>),
    Empty,
}

impl<'ctx> Iterator for Frames<'ctx> {
    type Item = Result<Frame, gimli::Error>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Frames::Actual(f) => f.next().transpose().map(|e| e.map(|b| Frame(b) )),
            Frames::Empty => None,
        }
    }
}

pub struct Frame(pub addr2line::Frame<EndianRcSlice<RunTimeEndian>>);

impl Frame {
    pub fn location(self) -> Option<Location> {
        self.0.location.map(|l| Location(l))
    }
}

impl fmt::Display for Frame {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut space = false;
        if let Some(func) = self.0.function.as_ref() {
            if let Ok(name) = func.demangle() {
                write!(f, "{}", name)?;
                space = true;
            }
        }
        if let Some(loc) = self.0.location.as_ref() {
            let unknown = String::from("???");
            if space {
                write!(f, " ({}:{})",
                    loc.file.as_ref().unwrap_or(&unknown),
                    loc.line.unwrap_or(0u64))?;
            } else {
                write!(f, "({}:{})",
                    loc.file.as_ref().unwrap_or(&unknown),
                    loc.line.unwrap_or(0u64))?;
            }
        }
        Ok(())
    }
}


pub struct Location(pub addr2line::Location);

impl PartialEq for Location {
    fn eq(&self, other: &Location) -> bool {
        self.0.file == other.0.file &&
        self.0.line == other.0.line &&
        self.0.column == other.0.column
    }
}

impl Eq for Location {}

impl fmt::Display for Location {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let unknown = String::from("???");
        write!(f, "{}:{}",
            self.0.file.as_ref().unwrap_or(&unknown),
            self.0.line.unwrap_or(0u64))
    }
}

impl fmt::Debug for Location {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Location{{file={:?}, line={:?}, column={:?} }}",
            self.0.file,
            self.0.line,
            self.0.column)
    }
}

#[derive(Fail, Debug)]
pub enum LookupError {
    #[fail(display = "Parse error {}", _0)]
    Object(String),
    #[fail(display = "{}", _0)]
    Gimli(#[cause] gimli::Error),
}

pub struct Lookup {
    pub name: PathBuf,
    pub info: DSYMInfo,
    pub uuid: Option<Uuid>,
    slide_addr: u64,
    addr2line: Context<EndianRcSlice<RunTimeEndian>>,
}

impl Lookup {
    fn load<'data>(name: PathBuf, info: DSYMInfo, macho: &mach::MachO<'data>) -> Result<Lookup, gimli::Error> {
        fn section_data_by_name<'data>(macho: &mach::MachO<'data>, section_name: &str) -> Option<Cow<'data, [u8]>>{
            let (system_section, section_name) = if section_name.starts_with('.') {
                (true, &section_name[1..])
            } else {
                (false, section_name)
            };
            let cmp_section_name = |name: Option<&str>| {
                name.map(|name| {
                    if system_section {
                        name.starts_with("__") && section_name == &name[2..]
                    } else {
                        section_name == name
                    }
                }).unwrap_or(false)
            };
            for segment in &macho.segments {
                for section in segment {
                    if let Ok((section, data)) = section {
                        if cmp_section_name(section.name().ok()) {
                            return Some(Cow::from(data));
                        }
                    } else {
                        break;
                    }
                }
            }
            None
        }

        fn mach_uuid<'data>(macho: &mach::MachO<'data>) -> Option<Uuid> {
            // Return the UUID from the `LC_UUID` load command, if one is present.
            macho
                .load_commands
                .iter()
                .filter_map(|lc| {
                    match lc.command {
                        CommandVariant::Uuid(ref cmd) => {
                            //TODO: Uuid should have a `from_array` method that can't fail.
                            Some(Uuid::from_bytes(cmd.uuid))
                        }
                        _ => None,
                    }
                }).nth(0)
        }

        fn slide<'data>(macho: &mach::MachO<'data>) -> Option<u64> {
            macho
                .load_commands
                .iter()
                .filter_map(|lc| {
                    match lc.command {
                        CommandVariant::Segment32(ref cmd) => {
                            //eprintln!("32bit Name {:?}", cmd.name());
                            if cmd.name().ok() == Some("__TEXT") {
                                Some(cmd.vmaddr as u64)
                            } else {
                                None
                            }
                        },
                        CommandVariant::Segment64(ref cmd) => {
                            //eprintln!("64bit Name {:?}", cmd.name());
                            if cmd.name().ok() == Some("__TEXT") {
                                Some(cmd.vmaddr)
                            } else {
                                None
                            }
                        },
                        _ => None,
                    }
                }).nth(0)
        }


        let endian = if macho.little_endian {
            gimli::RunTimeEndian::Little
        } else {
            gimli::RunTimeEndian::Big
        };

        fn load_section<'data, S, Endian>(macho: &mach::MachO<'data>, endian: Endian) -> S
        where
            S: gimli::Section<gimli::EndianRcSlice<Endian>>,
            Endian: gimli::Endianity,
        {
            let data = section_data_by_name(macho, S::section_name()).unwrap_or(Cow::Borrowed(&[]));
            S::from(gimli::EndianRcSlice::new(Rc::from(&*data), endian))
        }

        let uuid = mach_uuid(macho);
        eprintln!("UUID: {:?}", uuid);
        let debug_abbrev: gimli::DebugAbbrev<_> = load_section(macho, endian);
        let debug_info: gimli::DebugInfo<_> = load_section(macho, endian);
        let debug_line: gimli::DebugLine<_> = load_section(macho, endian);
        let debug_ranges: gimli::DebugRanges<_> = load_section(macho, endian);
        let debug_rnglists: gimli::DebugRngLists<_> = load_section(macho, endian);
        let debug_str: gimli::DebugStr<_> = load_section(macho, endian);

        let slide_addr = slide(macho).unwrap_or(0u64);

        let ctx = Context::from_sections(
            debug_abbrev,
            debug_info,
            debug_line,
            debug_ranges,
            debug_rnglists,
            debug_str,
        )?;
        Ok(Lookup {
            name: name,
            info: info,
            uuid: uuid,
            slide_addr: slide_addr,
            addr2line: ctx,
        })
    }

    pub fn find_location(&self, probe: u64) -> Result<Option<Location>, Error> {
        Ok(self.addr2line.find_location(self.slide_addr + probe)
            .context(ErrorKind::Probe(self.name.clone(), probe))?
            .map(|l| Location(l)))
    }

    pub fn find_frames(&self, probe: u64) -> Result<Frames, Error> {
        let frame_iter = self.addr2line
            .find_frames(self.slide_addr + probe)
            .context(ErrorKind::Probe(self.name.clone(), probe))?;
        Ok(Frames::Actual(frame_iter))
    }
}
