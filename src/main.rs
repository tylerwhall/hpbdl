extern crate byteorder;

use std::ffi::CString;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

use byteorder::{LittleEndian, ReadBytesExt};

const FILE_OFFSET_TABLE_START: u64 = 0x929;
const FILE_OFFSET_TABLE_ENTRY_SIZE: usize = 4;
const STRING_SIZE: u64 = 0x100;

fn vec_to_cstr(mut vec: Vec<u8>) -> String {
    let nul = vec.iter().position(|val| *val == 0).expect("nul");
    vec.resize(nul, 0);
    CString::new(vec).expect("cstring").into_string().expect(
        "cstr to str",
    )
}

fn read_string(file: &mut File) -> String {
    let mut string = Vec::with_capacity(STRING_SIZE as usize);
    file.by_ref()
        .take(STRING_SIZE)
        .read_to_end(&mut string)
        .expect("read string");
    vec_to_cstr(string)
}

// Name, offset, size
fn read_file_entry(file: &mut File) -> (String, u64, u64) {
    let name = read_string(file);
    let offset = file.read_u64::<LittleEndian>().unwrap();
    let size = file.read_u64::<LittleEndian>().unwrap();
    // Not sure if this is checksum
    let _checksum = file.read_u32::<LittleEndian>().unwrap();
    (name, offset, size)
}

/// File should point to file table entry. Returns offset.
fn extract_ipkg_file(file: &mut File, ipkg_start_offset: u64, dir: &str) -> u64 {
    let (name, offset, size) = read_file_entry(file);
    println!(" File: {} {:x} {:x}", name, offset, size);

    let mut outfile = File::create(Path::new(dir).join(name)).expect("create output");
    file.seek(SeekFrom::Start(ipkg_start_offset + offset))
        .expect("seek file");
    std::io::copy(&mut file.by_ref().take(size), &mut outfile).expect("copy");
    offset
}

fn split_ipkg(file: &mut File, index: usize) {
    const FILE_METADATA_SIZE: u64 = 0x14;
    const FILE_TABLE_ENTRY_SIZE: u64 = STRING_SIZE + FILE_METADATA_SIZE;
    const FILE_TABLE_START: u64 = 0x43d;

    let start = file.seek(SeekFrom::Current(0)).expect("seek ipkg");

    let mut header = [0; 4];
    file.read_exact(&mut header).expect("Read header");;
    if header != *b"ipkg" {
        println!("ipkg {} Header magic invalid", index);
        return;
    }
    // Name is at 0x220. Compensate for the 4 already read.
    file.seek(SeekFrom::Start(start + 0x220)).expect(
        "seek ipkg",
    );
    let name = read_string(file);
    println!("ipkg: {}", name);
    let dir = format!("{}.ipk", name);
    std::fs::create_dir_all(&dir).expect("create ipkg dir");
    file.seek(SeekFrom::Start(start + FILE_TABLE_START))
        .expect("seek ipkg");

    let offset = extract_ipkg_file(file, start, &dir);
    let len = offset - FILE_TABLE_START;
    let entries = len / FILE_TABLE_ENTRY_SIZE;
    println!("Size {} entries {}", len, entries);
    for i in 1..entries {
        file.seek(SeekFrom::Start(start + FILE_TABLE_START + FILE_TABLE_ENTRY_SIZE * i))
            .expect("seek file");
        extract_ipkg_file(file, start, &dir);
    }
}

fn main() {
    let file = std::env::args().skip(1).next().expect(
        "First arg is file name",
    );
    let mut file = File::open(file).expect("opening file");
    let mut header = [0; 4];
    file.read_exact(&mut header).expect("Read header");;
    if header != *b"ibdl" {
        println!("Header magic invalid");
        return;
    }

    file.seek(SeekFrom::Start(FILE_OFFSET_TABLE_START)).expect(
        "seek",
    );
    let first_start = file.read_u64::<LittleEndian>().unwrap();
    let first_size = file.read_u64::<LittleEndian>().unwrap();
    println!("First start 0x{:x}", first_start);
    println!("First size 0x{:x}", first_size);
    let len = first_start - FILE_OFFSET_TABLE_START;
    let entries = len / 16;
    println!("Size {} entries {}", len, entries);
    let mut table = Vec::with_capacity(entries as usize);
    table.push((first_start, first_size));
    for _ in 0..entries - 1 {
        let start = file.read_u64::<LittleEndian>().unwrap();
        let size = file.read_u64::<LittleEndian>().unwrap();
        println!("0x{:x} 0x{:x}", start, size);
        table.push((start, size));
    }

    for (i, entry) in table.iter().enumerate() {
        file.seek(SeekFrom::Start(entry.0)).expect("seek ipkg");
        split_ipkg(&mut file, i);
    }
}
