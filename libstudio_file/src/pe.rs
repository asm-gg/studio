use crate::file::FileHandle;
use core::ops::Range;
use pelite::{PeFile, Result, Wrap};

fn rva_to_offset(pe: &PeFile, rva: u32) -> Result<usize> {
    match pe {
        Wrap::T32(pe) => {
            use pelite::pe32::Pe;
            pe.rva_to_file_offset(rva)
        }
        Wrap::T64(pe) => {
            use pelite::pe64::Pe;
            pe.rva_to_file_offset(rva)
        }
    }
}

async fn dump_exports<'a>(pe: &PeFile<'a>) -> Result<Vec<Export<'a>>> {
    let mut res = Vec::new();
    match pe.exports() {
        Ok(exports) => {
            let by = exports.by()?;

            // TODO: exports by ordinal, get rid of the unwrap
            for result in by.iter_names() {
                if let (Ok(name), Ok(pelite::pe32::exports::Export::Symbol(addr))) = result {
                    res.push(Export {
                        name: name.to_str().unwrap(),
                        addr: Address::new(*addr, pe)?,
                    });
                }
            }
            Ok(vec![])
        }
        Err(pelite::Error::Null) => Ok(res),
        Err(e) => Err(e),
    }
}

pub struct Section<'a> {
    pub name: &'a str,
    pub bytes: &'a [u8],
    pub virtual_range: Range<u32>,
    pub executable: bool,
    pub writeable: bool,
}

pub struct Address {
    pub offset: usize,
    pub rva: u32,
}

pub struct Export<'a> {
    pub name: &'a str,
    pub addr: Address,
}

pub struct Pe<'a> {
    pub base: u64,
    pub headers: &'a [u8],
    pub sections: Vec<Section<'a>>,

    pub entry: Address,
    pub exports: Vec<Export<'a>>,
}

impl Address {
    pub fn new(rva: u32, pe: &PeFile) -> Result<Address> {
        Ok(Address {
            rva: rva,
            offset: rva_to_offset(pe, rva)?,
        })
    }
}

impl<'a> Section<'a> {
    pub fn new(sec: &'a pelite::image::IMAGE_SECTION_HEADER, file: &'a FileHandle) -> Section<'a> {
        let raw = sec.raw_range();
        // TODO: handle this shit...
        let name = sec.Name.to_str().expect("invalid utf8 section name");
        Section {
            name: name,
            bytes: &file.data[raw.start as usize..raw.end as usize],
            virtual_range: sec.virtual_range(),
            executable: sec.Characteristics & pelite::image::IMAGE_SCN_MEM_EXECUTE != 0,
            writeable: sec.Characteristics & pelite::image::IMAGE_SCN_MEM_WRITE != 0,
        }
    }
}

impl<'a> Pe<'a> {
    pub async fn parse(file: &'a FileHandle) -> Result<Pe<'a>> {
        let pe = PeFile::from_bytes(&file.data[..])?;
        let (entry, base, headers_size) = match pe.optional_header() {
            Wrap::T32(h) => (h.AddressOfEntryPoint, h.ImageBase as u64, h.SizeOfHeaders),
            Wrap::T64(h) => (h.AddressOfEntryPoint, h.ImageBase, h.SizeOfHeaders),
        };

        let mut me = Self {
            base: base,
            entry: Address::new(entry, &pe)?,
            headers: &file.data[..headers_size as usize],
            sections: vec![],
            exports: dump_exports(&pe).await?,
        };

        for section in pe.section_headers() {
            me.sections.push(Section::new(section, file));
        }

        Ok(me)
    }
}
