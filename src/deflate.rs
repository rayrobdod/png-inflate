///! A program that takes a png file and deflates the compressed chunks
///!
///! Reads stdin; writes to stdout

mod png;

use ::std::io::stdin;
use ::std::io::stdout;
use ::std::result::Result;

fn main() {
	let mut input = stdin();
	let mut output = stdout();

	match png::read(&mut input) {
		Result::Ok(indata) => {
			let outdata:Vec<png::Chunk> = indata.iter().map(|x| transform(x)).collect();

			match png::write(&mut output, outdata) {
				Result::Ok(()) => {
					// Ok
				}
				Result::Err(x) => {
					eprintln!("Could not write: {}", x);
					::std::process::exit(1);
				}
			}
		},
		Result::Err(x) => {
			eprintln!("Could not read: {}", x);
			::std::process::exit(1);
		},
	}
}

fn transform(indata: &png::Chunk) -> png::Chunk {
	match indata.typ {
		_ => indata.clone()
	}
}
