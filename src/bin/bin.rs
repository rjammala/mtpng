// CLI utility for testing the mtpng parallel PNG encoder
// by Brion Vibber <brion@pobox.com>
// 2018-09-03

use std::fs::File;
use std::io;
use std::io::{Error, ErrorKind};

// CLI options
extern crate clap;
use clap::{Arg, App, ArgMatches};

// For reading an existing file
extern crate png;

// Hey that's us!
extern crate mtpng;
use mtpng::{ColorType, CompressionLevel, Encoder, Header, Options};

pub fn err(payload: &str) -> Error
{
    Error::new(ErrorKind::Other, payload)
}

fn read_png(filename: &str) -> io::Result<(Header, Vec<u8>)> {
    let decoder = png::Decoder::new(File::open(filename)?);
    let (info, mut reader) = decoder.read_info()?;

    let header = Header::with_depth(info.width,
                                    info.height,
                                    ColorType::from_u8(info.color_type as u8)?,
                                    info.bit_depth as u8);
    let mut data = vec![0u8; info.buffer_size()];
    reader.next_frame(&mut data)?;

    Ok((header, data))
}

fn write_png(filename: &str, header: Header, options: Options, data: &[u8]) -> io::Result<()> {
    let writer = try!(File::create(filename));
    let mut encoder = Encoder::new(header, options, writer);

    encoder.write_header()?;
    for i in 0 .. header.height as usize {
        let start = header.stride() * i;
        let end = start + header.stride();
        encoder.append_row(&data[start .. end])?;
    }
    encoder.finish()?;

    Ok(())
}

fn doit(matches: ArgMatches) -> io::Result<()> {
    let infile = matches.value_of("input").unwrap();
    let outfile = matches.value_of("output").unwrap();
    let chunk_size = matches.value_of("chunk_size");
    let level = matches.value_of("level");

    let mut options = Options::default();
    match chunk_size {
        Some(s) => {
            options.chunk_size = s.parse::<usize>().unwrap()
        },
        None => {},
    }
    match level {
        Some(s) => {
            if s == "1" {
                options.compression_level = CompressionLevel::Fast
            } else if s == "9" {
                options.compression_level = CompressionLevel::High
            }
        },
        None => {},
    }

    println!("{} -> {}", infile, outfile);
    let (header, data) = read_png(&infile)?;
    write_png(&outfile, header, options, &data)?;
    Ok(())
}

pub fn main() {
    let matches = App::new("mtpng parallel PNG encoder")
        .version("0.1.0")
        .author("Brion Vibber <brion@pobox.com>")
        .about("Re-encodes PNG images using multiple CPU cores to exercise the mtpng library.")
        .arg(Arg::with_name("chunk-size")
            .long("chunk-size")
            .value_name("bytes")
            .help("Divide image into chunks of at least this given size.")
            .takes_value(true))
        .arg(Arg::with_name("level")
            .long("level")
            .value_name("level")
            .help("Set deflate compression level, from 1-9."))
        .arg(Arg::with_name("input")
            .help("Input filename, must be another PNG.")
            .required(true)
            .index(1))
        .arg(Arg::with_name("output")
            .help("Output filename.")
            .required(true)
            .index(2))
        .get_matches();

    match doit(matches) {
        Ok(()) => {},
        Err(e) => eprintln!("Error: {}", e),
    }
}