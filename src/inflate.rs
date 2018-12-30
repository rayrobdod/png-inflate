///! A program that takes a png file and deflates the compressed chunks

mod png;
mod zlib;

use ::std::result::Result;

fn main() {
	let args = ::std::env::args().fold(Args::default(), |fold, item| fold.push(&item));

	if args.help {
		Args::print_usage(& args.program_name.unwrap_or("".to_string()));
		::std::process::exit(0);
	}

	let mut input = FileOrStdin::from(args.input_file);
	let mut output = FileOrStdout::from(args.output_file);
	let ignore_unsafe_to_copy = args.ignore_unsafe_to_copy;

	match png::read(&mut input) {
		Result::Ok(indata) => {
			let outdata:Result<Vec<png::Chunk>, Error> = indata.iter().cloned().concat_idats().map(|x| deflate_chunks(x, ignore_unsafe_to_copy)).collect();

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
	UnsupportedCompressionMethod,
	ZlibError(zlib::InflateError),
}

impl From<zlib::InflateError> for Error {
	fn from(src: zlib::InflateError) -> Error { Error::ZlibError(src) }
}


fn deflate_chunks(indata:png::Chunk, ignore_unsafe_to_copy:bool) -> Result<png::Chunk, Error> {
	match indata.typ.as_ref() {
		// Not compressed, but contains data that can be validated
		b"IHDR" => {
			// byte 10 is the Compression Method; everything about this program assumes
			// that the only valid compression method is zero
			if indata.data[10] == 0 {
				Ok(indata)
			} else {
				Err(Error::UnsupportedCompressionMethod)
			}
		},
		// Contains only compressed data
		b"IDAT" => {
			Ok(png::Chunk{
				typ : *b"IDAT",
				data : zlib::deflate_immediate(&zlib::inflate(&indata.data)?),
			})
		},
		// Contains a cstring, followed by a method flag, followed by compressed data
		b"zTXt" | b"iCCP" => {
			let mut iter = indata.data.iter().cloned();
			let keyword:Vec<u8> = iter.by_ref().take_while(|x| *x != 0).collect();
			let method = iter.next();
			if Some(0) == method {
				let value:Vec<u8> = iter.collect();
				let value = zlib::deflate_immediate(&zlib::inflate(&value)?);
				let newdata = keyword.iter().chain([0, 0].iter()).chain(value.iter()).cloned().collect();
				Ok(png::Chunk{ typ : indata.typ, data : newdata, })
			} else {
				Err(Error::UnsupportedCompressionMethod)
			}
		},
		// Contains a: cstring, byte flag, byte flag, cstring, cstring, compressed data
		b"iTXt" => {
			let mut iter = indata.data.iter().cloned();
			let keyword:Vec<u8> = iter.by_ref().take_while(|x| *x != 0).collect();
			let is_compressed = iter.next();
			if Some(0) == is_compressed {
				// Not compressed, so make no changes
				let newdata:Vec<u8> = keyword.iter().cloned().chain(std::iter::repeat(0).take(2)).chain(iter).collect();
				Ok(png::Chunk{ typ : indata.typ, data : newdata, })
			} else {
				let method = iter.next();
				if Some(0) == method {
					let language:Vec<u8> = iter.by_ref().take_while(|x| *x != 0).collect();
					let translated_keyword:Vec<u8> = iter.by_ref().take_while(|x| *x != 0).collect();
					let value:Vec<u8> = iter.collect();
					let value = zlib::deflate_immediate(&zlib::inflate(&value)?);

					let newdata:Vec<u8> = keyword.iter().cloned().chain([0, 1, 0].iter().cloned())
							.chain(language.iter().cloned()).chain(std::iter::once(0))
							.chain(translated_keyword.iter().cloned()).chain(std::iter::once(0))
							.chain(value.iter().cloned()).collect();
					Ok(png::Chunk{ typ : indata.typ, data : newdata, })
				} else {
					Err(Error::UnsupportedCompressionMethod)
				}
			}
		},
		// Contain no compression, and are not affected by compression details of other chunks
		b"PLTE" | b"IEND" | b"tRNS" | b"cHRM" | b"gAMA" |
		b"sBIT" | b"sRGB" | b"tEXt" | b"bKGD" | b"hIST" |
		b"pHYs" | b"sPLT" | b"tIME" | b"oFFs" | b"pCAL" |
		b"sCAL" | b"gIFg" | b"gIFx" | b"gIFg" | b"eXIf" => Ok(indata),
		// unknown chunks
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

trait IteratorExt {
	fn concat_idats(self) -> ConcatinateIdats<Self> where Self:Sized + Iterator<Item=png::Chunk>;
}
impl <I: Sized + Iterator<Item=png::Chunk>> IteratorExt for I {
	fn concat_idats(self) -> ConcatinateIdats<I> where Self:Sized + Iterator<Item=png::Chunk> { ConcatinateIdats::new(self) }
}



#[derive(Debug, Default)]
struct Args {
	force_positional:bool,

	help:bool,
	ignore_unsafe_to_copy:bool,

	program_name:Option<String>,
	input_file:Option<String>,
	output_file:Option<String>,
}

impl Args {
	fn print_usage(program_name:&str) -> () {
		println!("  {0} [OPTIONS] [--] infile.png [outfile.png]", program_name);
		println!("  {0} [OPTIONS] < infile.png > outfile.png", program_name);
		println!("  {0} --help|-?", program_name);
		println!();
		println!("Decompresses a png image's internal data structures");
		println!();
		println!("  {:3} {:20} {}", "", "--copy-unsafe", "continue even upon encounter of unknown not-safe-to-copy chunks");
		println!("  {:3} {:20} {}", "-?,", "--help", "display this help message");
	}

	fn push(mut self, arg:&str) -> Args {
		let arg_zeroth_char = arg.chars().nth(0).unwrap_or('\0');
		if !self.force_positional && arg_zeroth_char == '-' {
			if arg == "--" {
				self.force_positional = true;
			} else if arg == "--copy-unsafe" || arg == "/copy-unsafe" {
				self.ignore_unsafe_to_copy = true;
			} else if arg == "-?" || arg == "--help" || arg == "/?" || arg == "/help" {
				self.help = true;
			} else {
				panic!(format!("Unknown flag: {}", arg));
			}
		} else {
			if self.program_name == Option::None {
				self.program_name = Option::Some(arg.to_string());
			} else if self.input_file == Option::None {
				self.input_file = Option::Some(arg.to_string());
			} else if self.output_file == Option::None {
				self.output_file = Option::Some(arg.to_string());
			} else {
				panic!("Too many positional arguments");
			}
		}
		self
	}
}


#[derive(Debug)]
enum FileOrStdin {
	File(::std::fs::File),
	Stdin(::std::io::Stdin),
}

impl From<Option<String>> for FileOrStdin {
	fn from(src:Option<String>) -> FileOrStdin {
		match src {
			None => FileOrStdin::Stdin(::std::io::stdin()),
			Some(s) => 	FileOrStdin::File(::std::fs::OpenOptions::new().read(true).open(s).expect("Could not open input file")),
		}
	}
}

impl ::std::io::Read for FileOrStdin {
	fn read(&mut self, buf: &mut [u8]) -> ::std::io::Result<usize> {
		match self {
			FileOrStdin::File(x) => x.read(buf),
			FileOrStdin::Stdin(x) => x.read(buf),
		}
	}
}

#[derive(Debug)]
enum FileOrStdout {
	File(::std::fs::File),
	Stdout(::std::io::Stdout),
}

impl From<Option<String>> for FileOrStdout {
	fn from(src:Option<String>) -> FileOrStdout {
		match src {
			None => FileOrStdout::Stdout(::std::io::stdout()),
			Some(s) => 	FileOrStdout::File(::std::fs::OpenOptions::new().write(true).create(true).open(s).expect("Could not open output file")),
		}
	}
}

impl ::std::io::Write for FileOrStdout {
	fn write(&mut self, buf: &[u8]) -> ::std::io::Result<usize> {
		match self {
			FileOrStdout::File(x) => x.write(buf),
			FileOrStdout::Stdout(x) => x.write(buf),
		}
	}
	fn flush(&mut self) -> ::std::io::Result<()> {
		match self {
			FileOrStdout::File(x) => x.flush(),
			FileOrStdout::Stdout(x) => x.flush(),
		}
	}
}


#[cfg(test)]
mod tests {
	mod concatinate_idats {
		use super::super::png;
		use super::super::IteratorExt;

		#[test]
		fn concat() {
			let data = [
				png::Chunk{typ : *b"IDAT", data: b"12345".to_vec()},
				png::Chunk{typ : *b"IDAT", data: b"6789A".to_vec()},
			];
			let mut dut = data.iter().cloned().concat_idats();
			assert_eq!(png::Chunk{typ : *b"IDAT", data: b"123456789A".to_vec()}, dut.next().unwrap());
			assert!(dut.next().is_none());
		}
	}
}
