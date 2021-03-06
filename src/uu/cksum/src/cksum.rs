// This file is part of the uutils coreutils package.
//
// (c) Michael Gehring <mg@ebfe.org>
//
//  For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

// spell-checker:ignore (ToDO) fname

#[macro_use]
extern crate uucore;

use std::fs::File;
use std::io::{self, stdin, BufReader, Read};
use std::path::Path;

include!(concat!(env!("OUT_DIR"), "/crc_table.rs"));

static SYNTAX: &str = "[OPTIONS] [FILE]...";
static SUMMARY: &str = "Print CRC and size for each file";
static LONG_HELP: &str = "";

#[inline]
fn crc_update(crc: u32, input: u8) -> u32 {
    (crc << 8) ^ CRC_TABLE[((crc >> 24) as usize ^ input as usize) & 0xFF]
}

#[inline]
fn crc_final(mut crc: u32, mut length: usize) -> u32 {
    while length != 0 {
        crc = crc_update(crc, length as u8);
        length >>= 8;
    }

    !crc
}

fn init_byte_array() -> Vec<u8> {
    vec![0; 1024 * 1024]
}

#[inline]
fn cksum(fname: &str) -> io::Result<(u32, usize)> {
    let mut crc = 0u32;
    let mut size = 0usize;

    let file;
    let mut rd: Box<dyn Read> = match fname {
        "-" => Box::new(stdin()),
        _ => {
            file = File::open(&Path::new(fname))?;
            Box::new(BufReader::new(file))
        }
    };

    let mut bytes = init_byte_array();
    loop {
        match rd.read(&mut bytes) {
            Ok(num_bytes) => {
                if num_bytes == 0 {
                    return Ok((crc_final(crc, size), size));
                }
                for &b in bytes[..num_bytes].iter() {
                    crc = crc_update(crc, b);
                }
                size += num_bytes;
            }
            Err(err) => return Err(err),
        }
    }
    //Ok((0 as u32,0 as usize))
}

pub fn uumain(args: impl uucore::Args) -> i32 {
    let matches = app!(SYNTAX, SUMMARY, LONG_HELP).parse(args.collect_str());

    let files = matches.free;

    if files.is_empty() {
        match cksum("-") {
            Ok((crc, size)) => println!("{} {}", crc, size),
            Err(err) => {
                show_error!("{}", err);
                return 2;
            }
        }
        return 0;
    }

    let mut exit_code = 0;
    for fname in &files {
        match cksum(fname.as_ref()) {
            Ok((crc, size)) => println!("{} {} {}", crc, size, fname),
            Err(err) => {
                show_error!("'{}' {}", fname, err);
                exit_code = 2;
            }
        }
    }

    exit_code
}
