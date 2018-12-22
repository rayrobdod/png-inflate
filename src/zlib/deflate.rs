use super::u4;
use super::Bits;

/// The extra bits following a length code to store
/// the actual value of the code
const LENGTH_EXTRA_BITS:[u4;29] = [
	u4::_0, u4::_0, u4::_0, u4::_0, u4::_0,
	u4::_0, u4::_0, u4::_0, u4::_1, u4::_1,
	u4::_1, u4::_1, u4::_2, u4::_2, u4::_2,
	u4::_2, u4::_3, u4::_3, u4::_3, u4::_3,
	u4::_4, u4::_4, u4::_4, u4::_4, u4::_5,
	u4::_5, u4::_5, u4::_5, u4::_0
];

/// The extra bits following a distance code to store
/// the actual value of the code
const DISTANCE_EXTRA_BITS:[u4;30] = [
	u4::_0, u4::_0, u4::_0, u4::_0, u4::_1,
	u4::_1, u4::_2, u4::_2, u4::_3, u4::_3,
	u4::_4, u4::_4, u4::_5, u4::_5, u4::_6,
	u4::_6, u4::_7, u4::_7, u4::_8, u4::_8,
	u4::_9, u4::_9, u4::_A, u4::_A, u4::_B,
	u4::_B, u4::_C, u4::_C, u4::_D, u4::_D,
];

/// An error that can occur while inflating a stream
#[derive(Debug)]
pub enum InflateError{
	UnexpectedEof,
	NonCompressedLengthInvalid,
	InvalidBtype,
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

/// Decompress the input deflate stream
pub fn inflate<I: Iterator<Item=u8>>(input:&mut I) -> Result<Vec<u8>, InflateError> {
	let mut bitreader = Bits::new(input);
	let mut retval:Vec<u8> = Vec::new();
	let mut read_final_chunk:bool = false;

	while !read_final_chunk {
		read_final_chunk = option_to_result(bitreader.next())?;
		let typ:u16 = option_to_result(bitreader.read_n_rev(u4::_2))?;
		match typ {
			0 => { // no compression
				let mut bytes = bitreader.discard_til_byte_boundary();
				let len = u8_concat_rev(option_to_result(bytes.next())?, option_to_result(bytes.next())?);
				let nlen = u8_concat_rev(option_to_result(bytes.next())?, option_to_result(bytes.next())?);
				if len != !nlen {
					return Err(InflateError::NonCompressedLengthInvalid);
				}
				for _ in 0..len {
					retval.push(option_to_result(bytes.next())?);
				}
				bitreader = Bits::new(bytes);
			}
			1 => { // fixed codes
				loop {
					let code = option_to_result(decode_fixed_huffman_code(&mut bitreader))?;
					eprintln!("{}", code);
					if code == 256 {
						break;
					} else if code < 256 {
						retval.push((code & 0xFF) as u8);
					} else {
						let length_index = code - 257;
						let length_extra_bits = LENGTH_EXTRA_BITS[usize::from(length_index)];
						let length:u16 = 3 + option_to_result(bitreader.read_n_rev(length_extra_bits))?
								+ LENGTH_EXTRA_BITS.iter().take(usize::from(length_index)).map(|x| x.nth_bit()).sum::<u16>();

						let distance_index = option_to_result(bitreader.read_n(u4::_5))?;
						let distance_extra_bits = DISTANCE_EXTRA_BITS[usize::from(distance_index)];
						let distance:u16 = 1 + option_to_result(bitreader.read_n(distance_extra_bits))?
								+ DISTANCE_EXTRA_BITS.iter().take(usize::from(distance_index)).map(|x| x.nth_bit()).sum::<u16>();

						eprintln!("\t{} {} ({})", length, distance, distance_index);
						for _ in 0..length {
							let index_to_copy = retval.len() - usize::from(distance);
							let value_to_copy = retval.get(index_to_copy).unwrap().clone();
							retval.push(value_to_copy);
						}
					}
				}
			}
			2 => { // custom codes
				panic!("TODO");
			}
			_ => { // error
				return Err(InflateError::InvalidBtype);
			}
		}
	}
	Ok(retval)
}

/// Store the the input in a deflate stream entirely using immediate mode (00)
pub fn deflate_immediate<I: Iterator<Item=u8>>(input:I) -> Vec<u8> {
	let input:Vec<u8> = input.collect();
	let length = input.len();
	let nlength = !length;
	// TODO: split into less-than-u16-sized chunks
	let length = u16_split_rev(length as u16);
	let nlength = u16_split_rev(nlength as u16);

	[1 as u8].iter()
			.chain(length.iter())
			.chain(nlength.iter())
			.chain(input.iter())
			.cloned().collect()
}

/// Decode a single a fixed-mode huffman code from the given stream
fn decode_fixed_huffman_code<I: Iterator<Item=u8>>(bitreader:&mut Bits<I>) -> Option<u16> {
	match bitreader.read_n(u4::_2)? {
		0 => {
			match bitreader.read_n(u4::_2)? {
				3 => bitreader.read_n(u4::_4),
				x => bitreader.read_n(u4::_3).map(|y| 256 + (x << 3) + y),
			}
		},
		1 => {
			bitreader.read_n(u4::_6).map(|y| 16 + y)
		},
		2 => {
			bitreader.read_n(u4::_6).map(|y| 80 + y)
		},
		3 => {
			match bitreader.read_n(u4::_3)? {
				0 => bitreader.read_n(u4::_3).map(|y| 280 + y),
				x => bitreader.read_n(u4::_4).map(|y| 144 + ((x - 1) << 4) + y),
			}
		},
		_ => panic!("")
	}
}

fn u8_concat_rev(a:u8, b:u8) -> u16 {
	u16::from(a) | (u16::from(b) << 8)
}

fn u16_split_rev(a:u16) -> [u8; 2] {
	[(a & 0xFF) as u8, ((a >> 8) & 0xFF) as u8]
}

#[cfg(test)]
mod tests {
	mod decode_fixed_huffman_code {
		use super::super::decode_fixed_huffman_code;

		#[test]
		fn hit_every_output() {
			let mut res:[bool;288] = [false;288];

			for i in u16::min_value()..u16::max_value() {
				let bits:[u8;2] = [((i >> 8) & 0xFF) as u8, (i & 0xFF) as u8];
				let bits = bits.iter().cloned();
				let mut bits = ::zlib::bits::Bits::new(bits);
				res[usize::from(decode_fixed_huffman_code(&mut bits).unwrap())] = true;
			}

			for i in 0..288 {
				assert!( res[i], "i = {}", i );
			}
		}
	}
	mod inflate {
		use super::super::inflate;
		#[test]
		fn immediate_mode() {
			let exp:[u8;10] = [1,2,3,4,5,6,7,8,9,10];
			let dut:[u8;15] = [1, 10, 0, !10, 0xFF, 1,2,3,4,5,6,7,8,9,10];
			let mut dut = dut.iter().cloned();
			let res = inflate(&mut dut).unwrap();
			assert!( exp == res.as_slice(), "{:?}", res );
		}
		#[test]
		fn one_zero() {
			let exp:[u8;1] = [0];
			let dut:[u8;3] = [0x63, 0x00, 0x00];
			let mut dut = dut.iter().cloned();
			let res = inflate(&mut dut).unwrap();
			assert!( exp == res.as_slice(), "{:?}", res );
		}
		#[test]
		fn four_zero() {
			let exp:[u8;4] = [0;4];
			let dut:[u8;4] = [0x63, 0x00, 0x02, 0x00];
			let mut dut = dut.iter().cloned();
			let res = inflate(&mut dut).unwrap();
			assert!( exp == res.as_slice(), "{:?}", res );
		}
		#[test]
		fn abcde_times_five() {
			let exp:[u8;25] = [0x61, 0x62, 0x63, 0x64, 0x65, 0x61, 0x62, 0x63, 0x64, 0x65, 0x61, 0x62, 0x63, 0x64, 0x65, 0x61, 0x62, 0x63, 0x64, 0x65, 0x61, 0x62, 0x63, 0x64, 0x65];
			let dut:[u8;9] = [0x4b, 0x4c, 0x4a, 0x4e, 0x49, 0xc5, 0x46, 0x00, 0x00];
			let mut dut = dut.iter().cloned();
			let res = inflate(&mut dut).unwrap();
			assert!( exp == res.as_slice(), "{:?}", res );
		}

		fn dfsa() {
			let exp:[u8;1] = [0];
			let dut:[u8;19] = [0x63, 0x64, 0x60, 0x60, 0xf8, 0x0f, 0xc4, 0x38, 0x01, 0x13, 0x94, 0xc6, 0x09, 0x86, 0x83, 0x02, 0x06, 0x06, 0x00];
			let mut dut = dut.iter().cloned();
			let res = inflate(&mut dut).unwrap();
			assert!( exp == res.as_slice(), "{:?}", res );
		}

		fn dfsa3() {
			let exp:[u8;1] = [0];
			let dut:[u8;22] = [0xe5, 0xca, 0x41, 0x0d, 0x00, 0x00, 0x00, 0x82, 0x40, 0xfb, 0x97, 0x56, 0x13, 0x50, 0x00, 0x36, 0x7e, 0x97, 0x57, 0x5a, 0x02, 0x06];
			let mut dut = dut.iter().cloned();
			let res = inflate(&mut dut).unwrap();
			assert!( exp == res.as_slice(), "{:?}", res );
		}
	}
}
