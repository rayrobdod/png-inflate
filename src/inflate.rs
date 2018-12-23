///! A program that takes a png file and deflates the compressed chunks
///!
///! Reads stdin; writes to stdout

mod png;
mod zlib;

use ::std::io::stdin;
use ::std::io::stdout;
use ::std::result::Result;

fn main() {
	let force:bool = false;
	let mut input = stdin();
	let mut output = stdout();

	match png::read(&mut input) {
		Result::Ok(indata) => {
			let outdata:Result<Vec<png::Chunk>, Error> = ConcatinateIdats::new(indata.iter().cloned()).map(|x| deflate_chunks(x, force)).collect();

			match outdata {
				Result::Ok(outdata) => {
					match png::write(&mut output, outdata) {
						Result::Ok(()) => {
							// Ok
						}
						Result::Err(x) => {
							eprintln!("Could not write: {}", x);
							::std::process::exit(1);
						}
					}
				}
				Result::Err(x) => {
					eprintln!("Could not transform: {:?}", x);
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

#[derive(Debug)]
pub enum Error{
	CannotCopySafely,
	ZlibError(zlib::InflateError),
}

impl From<zlib::InflateError> for Error {
	fn from(src: zlib::InflateError) -> Error { Error::ZlibError(src) }
}


fn deflate_chunks(indata:png::Chunk, ignore_unsafe_to_copy:bool) -> Result<png::Chunk, Error> {
	match indata.typ.as_ref() {
		// Validate, but copy if fine
		b"IHDR" => Ok(indata),
		// decompress
		b"IDAT" => {
			Ok(png::Chunk{
				typ : *b"IDAT",
				data : zlib::deflate_immediate(&zlib::inflate(&indata.data)?),
			})
		},
		b"zTXt" => Ok(indata),
		b"iTXt" => Ok(indata),
		b"iCCP" => Ok(indata),
		// known can copy
		b"PLTE" | b"IEND" | b"tRNS" | b"cHRM" | b"gAMA" |
		b"sBIT" | b"sRGB" | b"tEXt" | b"bKGD" | b"hIST" |
		b"pHYs" | b"sPLT" | b"tIME" => Ok(indata),
		// check safe to copy bit or loose override,
		// then either copy or error
		_ => {
			if ignore_unsafe_to_copy || indata.safe_to_copy() {
				Ok(indata)
			} else {
				Err(Error::CannotCopySafely)
			}
		}
	}
}

/// An iterator transformer that merges sequential IDATs, but otherwise passes through IDATs
struct ConcatinateIdats <I: Iterator<Item=png::Chunk>> {
	backing: ::std::iter::Peekable<I>,
}

impl <I: Iterator<Item=png::Chunk>> Iterator for ConcatinateIdats<I> {
	type Item = png::Chunk;
	fn next(&mut self) -> Option<png::Chunk> {
		match self.backing.next() {
			None => None,
			Some(sum) => {
				if sum.typ == *b"IDAT" {
					let mut retval_data = sum.data;
					while self.backing.peek().map(|x| x.typ) == Some(*b"IDAT") {
						retval_data.append(&mut self.backing.next().unwrap().data.clone());
					}
					Some(png::Chunk{
						typ : *b"IDAT",
						data: retval_data,
					})
				} else {
					Some(sum)
				}
			}
		}
	}
}

impl <I: Iterator<Item=png::Chunk>> ConcatinateIdats<I> {
	fn new(backing:I) -> ConcatinateIdats<I> { ConcatinateIdats {
		backing : backing.peekable()
	}}
}

#[cfg(test)]
mod tests {
	mod concatinate_idats {
		use super::super::png;
		use super::super::ConcatinateIdats;

		#[test]
		fn concat() {
			let data = [
				png::Chunk{typ : *b"IDAT", data: b"12345".to_vec()},
				png::Chunk{typ : *b"IDAT", data: b"6789A".to_vec()},
			];
			let mut dut = ConcatinateIdats::new(data.iter().cloned());
			assert_eq!(png::Chunk{typ : *b"IDAT", data: b"123456789A".to_vec()}, dut.next().unwrap());
			assert!(dut.next().is_none());
		}
	}
}
