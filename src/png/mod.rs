//! http://www.libpng.org/pub/png/

use ::std::io::ErrorKind;
use ::std::io::Read;
use ::std::io::Write;
use ::std::iter::Iterator;
use ::std::result::Result;
use ::std::vec::Vec;

/// The PNG magic header
const MAGIC:[u8;8] = [137, 80, 78, 71, 13, 10, 26, 10];


/// Reads a png file, and returns the chunks contained in that file
pub fn read(file:&mut dyn Read) -> Result<Vec<Chunk>, ReadError> {
	let mut magic:[u8;8] = [0,0,0,0,0,0,0,0];
	file.read_exact(&mut magic).map_err(ReadError::Io).and_then(|_| {
		let magic = magic;
		if magic == MAGIC {
			let mut retval:Vec<Chunk> = Vec::new();

			loop {
				match Chunk::read(file) {
					Result::Ok(Some(x)) => {
						retval.push(x);
					},
					Result::Ok(None) => {
						break Ok(retval);
					}
					Result::Err(x) => {
						break Err(ReadError::from(x));
					},
				}
			}
		} else {
			Err(ReadError::MagicMismatch(magic))
		}
	})
}

/// Writes a sequence of Chunks to form a png file
pub fn write(file:&mut dyn Write, chunks:Vec<Chunk>) -> Result<(), ::std::io::Error> {
	file.write_all(&MAGIC)?;
	for chunk in chunks {
		chunk.write(file)?;
	}
	Ok(())
}


/// Represents a PNG data chunk
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Chunk {
	pub typ:[u8;4],
	pub data:Vec<u8>
}

impl Chunk {
	/// Reads a PNG chunk from a data stream
	fn read(file:&mut dyn Read) -> Result<Option<Chunk>, ChunkReadError> {
		let mut size:[u8;4] = [0,0,0,0];
		let (mut size_head, mut size_tail) = size.split_at_mut(1);
		if let Err(e) = file.read_exact(&mut size_head) {
			if e.kind() == ErrorKind::UnexpectedEof {
				return Ok(None)
			} else {
				return Err(ChunkReadError::Io(e))
			}
		}
		file.read_exact(&mut size_tail).map_err(ChunkReadError::Io).and_then(|_| {
			let size = u32::from_be_bytes(size);

			let mut typ:[u8;4] = [0,0,0,0];
			file.read_exact(&mut typ).map_err(ChunkReadError::Io).and_then(|_| {
				let typ = typ;

				if ! typ.iter().all(|c| (0x41 <= *c && *c <= 0x5A) || (0x61 <= *c && *c <= 0x7A)) {
					Err(ChunkReadError::InvalidTyp(typ))
				} else {
					let mut data:Vec<u8> = vec![0; u32_to_usize(size)];
					file.read_exact(&mut data).map_err(ChunkReadError::Io).and_then(|_| {
						let data = data;

						let mut stated_crc:[u8;4] = [0; 4];
						file.read_exact(&mut stated_crc).map_err(ChunkReadError::Io).and_then(|_| {
							let stated_crc = u32::from_be_bytes(stated_crc);
							let cacluated_crc = calculate_crc(typ.iter().chain(data.iter()));

							if stated_crc == cacluated_crc {
								Ok(Some(Chunk{typ, data}))
							} else {
								Err(ChunkReadError::CrcMismatch{stated:stated_crc, calculated:cacluated_crc})
							}
						})
					})
				}
			})
		})
	}

	/// Writes a PNG chunk to a data stream
	fn write(self, file:&mut dyn Write) -> Result<(), ::std::io::Error> {
		file.write_all(&(self.data.len() as u32).to_be_bytes())?;
		file.write_all(&self.typ)?;
		file.write_all(&self.data)?;
		file.write_all(&(calculate_crc(self.typ.iter().chain(self.data.iter()))).to_be_bytes())?;
		Ok(())
	}

	/// Returns whether the chunk type is safe to copy without knowing what it is
	pub fn safe_to_copy(&self) -> bool {
		0 != (self.typ[3] & 0x20)
	}
}


/// Represents an error that can occur when decoding a PNG Chunk
#[derive(Debug)]
pub enum ReadError{
	/** An IO error */
	Io(::std::io::Error),
	/** The chunk's typ is invalid (a byte was outside the range `A-Za-z`) */
	InvalidTyp([u8;4]),
	/** The calculated CRC did not match the given CRC */
	CrcMismatch{stated:u32, calculated:u32},
	/** The given magic header didn't match the expected PNG header */
	MagicMismatch([u8;8]),
}

impl ::std::fmt::Display for ReadError {
	fn fmt(&self, f:&mut ::std::fmt::Formatter) -> ::std::fmt::Result {
		match self {
			ReadError::Io(x) => write!(f, "{}", x),
			ReadError::InvalidTyp(x) => write!(f, "Illegal chunk type: {:?}", x),
			ReadError::CrcMismatch{stated, calculated} => write!(f, "CRC mismatch: file {:x}; calculated {:x}", stated, calculated),
			ReadError::MagicMismatch(x) => {
				let bytes = x;
				let chars:String = x.iter().map(|x| char::from(*x)).map(|x| if x.is_ascii_graphic() {x} else {'.'}).collect();
				write!(f, "Magic didn't match expected: {:?} | {:?}", bytes, chars)
			},
		}
	}
}

impl From<ChunkReadError> for ReadError {
	fn from(src:ChunkReadError) -> ReadError {
		match src {
			ChunkReadError::Io(x) => ReadError::Io(x),
			ChunkReadError::InvalidTyp(x) => ReadError::InvalidTyp(x),
			ChunkReadError::CrcMismatch{stated, calculated} => ReadError::CrcMismatch{stated, calculated},
		}
	}
}

/// Represents an error that can occur when decoding a PNG Chunk
#[derive(Debug)]
pub enum ChunkReadError{
	/** An IO error */
	Io(::std::io::Error),
	/** The chunk's typ is invalid (a byte was outside the range `A-Za-z`) */
	InvalidTyp([u8;4]),
	/** The calculated CRC did not match the given CRC */
	CrcMismatch{stated:u32, calculated:u32},
}

impl ::std::fmt::Display for ChunkReadError {
	fn fmt(&self, f:&mut ::std::fmt::Formatter) -> ::std::fmt::Result {
		match self {
			ChunkReadError::Io(x) => write!(f, "{}", x),
			ChunkReadError::InvalidTyp(x) => write!(f, "Illegal chunk type: {:?}", x),
			ChunkReadError::CrcMismatch{stated, calculated} => write!(f, "CRC mismatch: file {:x}; calculated {:x}", stated, calculated),
		}
	}
}


#[cfg(target_pointer_width = "32")]
fn u32_to_usize(a:u32) -> usize {
	a as usize
}

#[cfg(target_pointer_width = "64")]
fn u32_to_usize(a:u32) -> usize {
	a as usize
}

fn calculate_crc<'a, I: IntoIterator<Item=&'a u8>>(buffer:I) -> u32 {
	const CRC_POLYNOMIAL:u32 = 0xedb8_8320;
	fn update_crc(crc:u32, message:u8) -> u32 {
		let message:u32 = u32::from(message);
		let mut crc = crc ^ message;
		for _ in 0..8 {
			crc = (if crc & 1 != 0 {CRC_POLYNOMIAL} else {0}) ^ (crc >> 1);
		}
		crc
	}

	buffer.into_iter()
		.fold(u32::max_value(), |crc, message| update_crc(crc, *message))
		^ u32::max_value()
}


#[cfg(test)]
mod tests {
	mod calculate_crc {
		use super::super::calculate_crc;

		#[test]
		fn nul() {
			let val:[u8;0] = [];
			let exp:u32 = 0;
			let res = calculate_crc(val.iter());
			assert!( exp == res, "{:x} != {:x}", exp, res );
		}

		#[test]
		fn iend() {
			let val:[u8;4] = [0x49, 0x45, 0x4e, 0x44];
			let exp:u32 = 0xae426082;
			let res = calculate_crc(&val);
			assert!( exp == res, "{:x} != {:x}", exp, res );
		}

		#[test]
		fn ihdr_1() {
			let val:[u8;17] = [
				0x49, 0x48, 0x44, 0x52, 0x00, 0x00, 0x01, 0x2c,
				0x00, 0x00, 0x00, 0x96, 0x02, 0x03, 0x00, 0x00,
				0x00
			];
			let exp:u32 = 0x19355d41;
			let res = calculate_crc(&val);
			assert!( exp == res, "{:x} != {:x}", exp, res );
		}
	}

	mod chunk_read {
		use super::super::Chunk;
		use super::super::ChunkReadError;
		use ::std::io::ErrorKind;

		fn assert_is_err_eof(e : Result<Option<Chunk>, ChunkReadError>) {
			if let Err(e) = e {
				if let ChunkReadError::Io(e) = e {
					if e.kind() == ErrorKind::UnexpectedEof {
						// success
					} else {
						panic!("Was error, but was not EOF");
					}
				} else {
					panic!("Was error, but was not EOF");
				}
			} else {
				panic!("Was not error");
			}
		}

		#[test]
		fn exact_size() {
			let exp = Some(Chunk{typ:*b"ABCD", data:vec![61, 62, 63, 64]});
			let mut dut:&[u8] = &[0, 0, 0, 4, 0x41, 0x42, 0x43, 0x44, 61, 62, 63, 64, 0x75, 0x88, 0x7C, 0x4B];
			let res = Chunk::read(&mut dut).unwrap();
			assert!( exp == res );
			assert!( dut.len() == 0 );
		}

		#[test]
		fn reads_only_the_amount_needed() {
			let exp = Some(Chunk{typ:*b"ABCD", data:vec![61, 62, 63, 64]});
			let mut dut:&[u8] = &[0, 0, 0, 4, 0x41, 0x42, 0x43, 0x44, 61, 62, 63, 64, 0x75, 0x88, 0x7C, 0x4B, 11, 22, 33, 44, 55];
			let res = Chunk::read(&mut dut).unwrap();
			assert!( exp == res );
			assert!( dut.len() == 5 );
		}

		#[test]
		fn errors_if_unexpected_eof_crc() {
			let mut dut:&[u8] = &[0, 0, 0, 4, 0x41, 0x42, 0x43, 0x44, 61, 62, 63, 64, 0x75, 0x88];
			let res = Chunk::read(&mut dut);
			assert_is_err_eof(res);
		}

		#[test]
		fn errors_if_unexpected_eof_data() {
			let mut dut:&[u8] = &[0, 0, 0, 4, 0x41, 0x42, 0x43, 0x44, 61, 62];
			let res = Chunk::read(&mut dut);
			assert_is_err_eof(res);
		}

		#[test]
		fn errors_if_unexpected_eof_typ() {
			let mut dut:&[u8] = &[0, 0, 0, 4, 0x41, 0x42];
			let res = Chunk::read(&mut dut);
			assert_is_err_eof(res);
		}

		#[test]
		fn errors_if_unexpected_eof_size() {
			let mut dut:&[u8] = &[0];
			let res = Chunk::read(&mut dut);
			assert_is_err_eof(res);
		}

		#[test]
		fn reports_valid_eof() {
			let exp = None;
			let mut dut:&[u8] = &[];
			let res = Chunk::read(&mut dut).unwrap();
			assert!( exp == res );
			assert!( dut.len() == 0 );
		}
	}

	mod file_read {
		use super::super::read;
		use super::super::Chunk;
		use super::super::ReadError;
		use ::std::io::ErrorKind;

		fn assert_is_err_eof(e : Result<Vec<Chunk>, ReadError>) {
			if let Err(e) = e {
				if let ReadError::Io(e) = e {
					if e.kind() == ErrorKind::UnexpectedEof {
						// success
					} else {
						panic!("Was error, but was not EOF");
					}
				} else {
					panic!("Was error, but was not EOF");
				}
			} else {
				panic!("Was not error");
			}
		}

		fn assert_is_err_magic(e : Result<Vec<Chunk>, ReadError>) {
			if let Err(e) = e {
				if let ReadError::MagicMismatch(_) = e {
					// success
				} else {
					panic!("Was error, but was not MagicMismatch");
				}
			} else {
				panic!("Was not error");
			}
		}

		#[test]
		fn normal_case() {
			let exp = vec![
				Chunk{typ:*b"FIRS", data:vec![]},
				Chunk{typ:*b"SECO", data:vec![]},
				Chunk{typ:*b"THIR", data:vec![]},
			];
			let mut dut:&[u8] = &[
				137, b'P', b'N', b'G', b'\r', b'\n', 26, b'\n',
				0, 0, 0, 0, b'F', b'I', b'R', b'S', 0x9A, 0x9F, 0x51, 0x2A,
				0, 0, 0, 0, b'S', b'E', b'C', b'O', 0xB3, 0x9A, 0x70, 0xBC,
				0, 0, 0, 0, b'T', b'H', b'I', b'R', 0xBF, 0x7C, 0x5F, 0x05,
			];
			let res = read(&mut dut).unwrap();
			assert!( exp == res );
			assert!( dut.len() == 0 );
		}

		#[test]
		fn does_not_treat_iend_specially() {
			let exp = vec![
				Chunk{typ:*b"FIRS", data:vec![]},
				Chunk{typ:*b"IEND", data:vec![]},
				Chunk{typ:*b"THIR", data:vec![]},
			];
			let mut dut:&[u8] = &[
				137, b'P', b'N', b'G', b'\r', b'\n', 26, b'\n',
				0, 0, 0, 0, b'F', b'I', b'R', b'S', 0x9A, 0x9F, 0x51, 0x2A,
				0, 0, 0, 0, b'I', b'E', b'N', b'D', 0xAE, 0x42, 0x60, 0x82,
				0, 0, 0, 0, b'T', b'H', b'I', b'R', 0xBF, 0x7C, 0x5F, 0x05,
			];
			let res = read(&mut dut).unwrap();
			assert!( exp == res );
			assert!( dut.len() == 0 );
		}

		#[test]
		fn incomplete_chunk_1() {
			let mut dut:&[u8] = &[
				137, b'P', b'N', b'G', b'\r', b'\n', 26, b'\n',
				0,
			];
			let res = read(&mut dut);
			assert_is_err_eof(res);
		}

		#[test]
		fn incomplete_chunk_2() {
			let mut dut:&[u8] = &[
				137, b'P', b'N', b'G', b'\r', b'\n', 26, b'\n',
				0, 0,
			];
			let res = read(&mut dut);
			assert_is_err_eof(res);
		}

		#[test]
		fn incorrect_magic() {
			let mut dut:&[u8] = &[
				138, b'M', b'N', b'G', b'\r', b'\n', 26, b'\n',
			];
			let res = read(&mut dut);
			assert_is_err_magic(res);
		}
	}

	mod chunk_safe_to_copy {
		use super::super::Chunk;

		#[test]
		fn tru() {
			let exp:bool = true;
			let res = Chunk{typ:*b"IDAt", data:vec![]}.safe_to_copy();
			assert!( exp == res );
		}
		#[test]
		fn fals() {
			let exp:bool = false;
			let res = Chunk{typ:*b"IDAT", data:vec![]}.safe_to_copy();
			assert!( exp == res );
		}
	}
}
