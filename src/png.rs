///! http://www.libpng.org/pub/png/

use ::std::io::Read;
use ::std::iter::Iterator;
use ::std::result::Result;
use ::std::vec::Vec;

/// The PNG magic header
pub const MAGIC:[u8;8] = [137, 80, 78, 71, 13, 10, 26, 10];


/// Reads a png file
pub fn read(file:&mut Read) -> Result<Vec<Chunk>, ReadError> {
	let mut magic:[u8;8] = [0,0,0,0,0,0,0,0];
	file.read_exact(&mut magic).map_err(|x| ReadError::Io(x)).and_then(|_| {
		let magic = magic;
		if magic == MAGIC {
			let mut retval:Vec<Chunk> = Vec::new();

			loop {
				match Chunk::read(file) {
					Result::Ok(x) => {
						let typ = &x.typ.clone();
						retval.push(x);
						if typ == b"IEND" {
							break Ok(retval);
						}
					},
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





/// Represents a PNG chunk
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Chunk {
	pub typ:[u8;4],
	pub data:Vec<u8>
}

impl Chunk {
	pub fn read(file:&mut Read) -> Result<Chunk, ChunkReadError> {
		let mut size:[u8;4] = [0,0,0,0];
		file.read_exact(&mut size).map_err(|x| ChunkReadError::Io(x)).and_then(|_| {
			let size = bytes_to_u32(size);
			
			let mut typ:[u8;4] = [0,0,0,0];
			file.read_exact(&mut typ).map_err(|x| ChunkReadError::Io(x)).and_then(|_| {
				let typ = typ;
				
				if ! typ.iter().all(|c| (0x41 <= *c && *c <= 0x5A) || (0x61 <= *c && *c <= 0x7A)) {
					Err(ChunkReadError::InvalidTyp(typ))
				} else {
					let mut data:Vec<u8> = vec![0; u32_to_usize(size)];
					file.read_exact(&mut data).map_err(|x| ChunkReadError::Io(x)).and_then(|_| {
						let data = data;
						
						let mut stated_crc:[u8;4] = [0; 4];
						file.read_exact(&mut stated_crc).map_err(|x| ChunkReadError::Io(x)).and_then(|_| {
							let stated_crc = bytes_to_u32(stated_crc);
							let cacluated_crc = calculate_crc(typ.iter().chain(data.iter()));
							
							if stated_crc == cacluated_crc {
								Ok(Chunk{typ, data})
							} else {
								Err(ChunkReadError::CrcMismatch{stated:stated_crc, calculated:cacluated_crc})
							}
						})
					})
				}
			})
		})
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
			ReadError::MagicMismatch(x) => write!(f, "Magic didn't match expected: {:?}", x),
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

fn bytes_to_u32(bytes:[u8;4]) -> u32 {
	bytes.iter().zip(0..)
		.map(|(byte, index)| u32::from(*byte) << ((3 - index) * 8))
		.sum()
}

const CRC_POLYNOMIAL:u32 = 0xedb8_8320;
fn update_crc(crc:u32, message:u8) -> u32 {
	let message:u32 = u32::from(message);
	let mut crc = crc ^ message;
	for _ in 0..8 {
		crc = (if crc & 1 != 0 {CRC_POLYNOMIAL} else {0}) ^ (crc >> 1);
	}
	crc
}

fn calculate_crc<'a, I: IntoIterator<Item=&'a u8>>(buffer:I) -> u32 {
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
}


