///! A program that reads a file and checks it for PNG validity

mod png;

use ::std::env::args;
use ::std::option::Option;
use ::std::result::Result;
use ::std::string::String;

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
		
		match png::read(&mut input_file) {
			Result::Ok(_) => {
				println!("OK");
			},
			Result::Err(x) => {
				println!("{}", x);
				::std::process::exit(1);
			},
		}
	} else {
		println!("Usage: {} input.png", program_name.unwrap_or("pngcheck".to_string()));
	}
}
