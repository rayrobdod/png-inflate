#![allow(dead_code)]
///! "ZLIB Compressed Data Format Specification" <https://www.ietf.org/rfc/rfc1950.txt>
///! "DEFLATE Compressed Data Format Specification" <http://www.w3.org/Graphics/PNG/RFC-1951>

mod u4mod;
pub use self::u4mod::u4;
pub use self::u4mod::ZeroToRangeIter as u4ZeroToRangeIter;
mod bits;
use self::bits::Bits;
mod deflate;

/// A u2 representing a hint indicating the algorithm used when compressing
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum CompressionLevel {
	Fastest, Fast, Slow, Slowest
}

impl From<CompressionLevel> for u8 {
	fn from(src:CompressionLevel) -> u8 {
		/*
		match src {
			CompressionLevel::Fastest => 0 << 6,
			CompressionLevel::Fast => 1 << 6,
			CompressionLevel::Slow => 2 << 6,
			CompressionLevel::Slowest => 3 << 6,
		}
		*/
		(src as u8) << 6
	}
}




/// Represents the zlib format header
///
/// Assumes that the compression_method is 8 and that has_dictionary is false.
struct Header {
	window_size_exponent:u4,
	compression_level:CompressionLevel
}

impl Header {
	fn new(window_size_exponent:u4, compression_level:CompressionLevel) -> Header {
		Header{window_size_exponent:window_size_exponent, compression_level:compression_level}
	}

	fn window_size(&self) -> u32 {
		1 << (8 + u8::from(self.window_size_exponent))
	}

	pub fn read(val:u16) -> Result<Header, InflateError> {
		if (val % 31) != 0 {
			Err(InflateError::CheckMismatchHeader)
		} else {
			let b1 = ((val >> 8) & 255) as u8;
			let b2 = (val & 255) as u8;
			let method = u4::truncate(b1);
			let info = u4::truncate(b1 >> 4);
			let _check = b2 & 31;
			let dict = 0 != (b2 & 32);
			let level = b2 >> 6;
			let level = match level {
				0 => CompressionLevel::Fastest,
				1 => CompressionLevel::Fast,
				2 => CompressionLevel::Slow,
				3 => CompressionLevel::Slowest,
				_ => panic!("")
			};

			if method != u4::_8 {
				Err(InflateError::UnsupportedHeader)
			} else if dict {
				Err(InflateError::UnsupportedHeader)
			} else {
				Ok(Header::new(info, level))
			}
		}
	}

	pub fn write(&self) -> u16 {
		let b1 = u4::concat(self.window_size_exponent, u4::_8);
		let b2 = u8::from(self.compression_level);
		let retval = (u16::from(b1) << 8) | u16::from(b2);
		// Set the check bits so that the output value is a multiple of 31
		let modulus = retval % 31;
		let retval = retval + (31 - modulus);
		assert!(retval % 31 == 0);
		//
		retval
	}
}


#[derive(Debug)]
pub enum InflateError{
	UnexpectedEof,
	CheckMismatchHeader,
	UnsupportedHeader,
	ChecksumMismatch,

	DeflateNonCompressedLengthInvalid,
	DeflateInvalidBtype,
}

impl From<deflate::InflateError> for InflateError {
	fn from(src: deflate::InflateError) -> InflateError { match src {
		deflate::InflateError::UnexpectedEof => InflateError::UnexpectedEof,
		deflate::InflateError::NonCompressedLengthInvalid => InflateError::DeflateNonCompressedLengthInvalid,
		deflate::InflateError::InvalidBtype => InflateError::DeflateInvalidBtype,
	}}
}

//impl From<::std::option::NoneError> for InflateError {
//	fn from(src: ::std::option::NoneError) -> InflateError { InflateError::UnexpectedEof }
//}
fn option_to_result<A>(a:Option<A>) -> Result<A, InflateError> {
	match a {
		Some(x) => Ok(x),
		None => Err(InflateError::UnexpectedEof),
	}
}

/// Decompresses a zlib stream
pub fn inflate(r : &[u8]) -> Result<Vec<u8>, InflateError> {
	let mut r = r.iter().cloned();
	let _header = Header::read(u8_concat(option_to_result(r.next())?, option_to_result(r.next())?))?;
	let result = deflate::inflate(&mut r)?;
	let given_chksum = bytes_to_u32([
		option_to_result(r.next())?,
		option_to_result(r.next())?,
		option_to_result(r.next())?,
		option_to_result(r.next())?,
	]);
	let calculated_chksum = adler32(&result);

	if given_chksum != calculated_chksum {
		eprintln!("{:x} {:x}", given_chksum, calculated_chksum);
		Err(InflateError::ChecksumMismatch)
	} else {
		Ok(result)
	}
}

/// Store the the input in a zlib stream entirely using immediate mode
pub fn deflate_immediate(r : &[u8]) -> Vec<u8> {
	u16_split(Header::new(u4::_7, CompressionLevel::Fastest).write()).iter().cloned()
			.chain(deflate::deflate_immediate(r.iter().cloned()))
			.chain(u32_to_bytes(adler32(r)).iter().cloned())
			.collect()
}


fn u8_concat(a:u8, b:u8) -> u16 {
	(u16::from(a) << 8) | u16::from(b)
}

fn u16_split(a:u16) -> [u8; 2] {
	[((a >> 8) & 0xFF) as u8, (a & 0xFF) as u8]
}


/// Computes an adler 32 checksum
fn adler32(input:&[u8]) -> u32 {
	const DIVISOR:u32 = 65521;
	let mut s1:u32 = 1;
	let mut s2:u32 = 0;

	for x in input {
		s1 += u32::from(*x);
		s1 %= DIVISOR;
		s2 += s1;
		s2 %= DIVISOR;
	}

	(s2 << 16) | s1
}


fn bytes_to_u32(bytes:[u8;4]) -> u32 {
	bytes.iter().zip(0..)
		.map(|(byte, index)| u32::from(*byte) << ((3 - index) * 8))
		.sum()
}

fn u32_to_bytes(a:u32) -> [u8;4] {
	[
		((a & 0xFF00_0000) >> 24) as u8,
		((a & 0xFF_0000) >> 16) as u8,
		((a & 0xFF00) >> 8) as u8,
		(a & 0xFF) as u8,
	]
}




#[cfg(test)]
mod tests {
	mod header_write {
		use super::super::Header;
		use super::super::u4;
		use super::super::CompressionLevel;

		#[test]
		fn default() {
			let exp:u16 = 0x6881;
			let dut:Header = Header::new(u4::_6, CompressionLevel::Slow);
			let res = dut.write();
			assert!( exp == res, "{:x} != {:x}", exp, res );
		}
		#[test]
		fn fastest() {
			let exp:u16 = 0x7801;
			let dut:Header = Header::new(u4::_7, CompressionLevel::Fastest);
			let res = dut.write();
			assert!( exp == res, "{:x} != {:x}", exp, res );
		}
	}
}
