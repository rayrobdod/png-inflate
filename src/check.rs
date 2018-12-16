///! A program that reads a file and checks it for PNG validity

mod png;

use ::std::env::args;
use ::std::io::Read;
use ::std::option::Option;
use ::std::result::Result;
use ::std::string::String;

const ERR_NO_MAGIC:&'static str = "Input file does not have PNG magic header";

fn main() {
	let mut program_name:Option<String> = Option::None;
	let mut input_file:Option<String> = Option::None;
	
	for arg in args() {
		if program_name == Option::None {
			program_name = Option::Some(arg);
		} else if input_file == Option::None {
			input_file = Option::Some(arg);
		}
	}
	
	let program_name = program_name;
	let input_file = input_file;
	
	if let Option::Some(input_file) = input_file {
		let mut read_open_option = std::fs::OpenOptions::new();
		read_open_option.read(true);
		let read_open_option = read_open_option;
		
		let mut input_file = read_open_option.open(input_file)
				.expect("base file does not exist");
		
		let mut magic:[u8;8] = [0,0,0,0,0,0,0,0];
		input_file.read_exact(&mut magic).expect(ERR_NO_MAGIC);
		if magic == png::MAGIC {
			
			loop {
				match png::Chunk::read(&mut input_file) {
					Result::Ok(x) => {
						if &x.typ == b"IEND" {
							println!("OK");
							break;
						}
					},
					Result::Err(x) => {
						println!("{}", x);
						::std::process::exit(1);
					},
				}
			}
			
		} else {
			println!("{}", ERR_NO_MAGIC);
			::std::process::exit(2);
		}
	} else {
		println!("Usage: {} input.png", program_name.unwrap_or("pngcheck".to_string()));
	}
}
