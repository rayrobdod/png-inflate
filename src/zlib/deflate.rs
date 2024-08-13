//! "DEFLATE Compressed Data Format Specification" <http://www.w3.org/Graphics/PNG/RFC-1951>
use super::u4;
use super::Bits;

#[rustfmt::skip]
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

#[rustfmt::skip]
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

#[rustfmt::skip]
/// The order that meta codes are stored in 10 mode codings
const META_CODES_ORDER:[usize;19] = [
	16, 17, 18, 0, 8,
	7, 9, 6, 10, 5,
	11, 4, 12, 3, 13,
	2, 14, 1, 15,
];

/// An error that can occur while inflating a stream
#[derive(Debug)]
pub enum InflateError {
	UnexpectedEof,
	NonCompressedLengthInvalid,
	InvalidBtype,
}

//impl From<::std::option::NoneError> for InflateError {
//	fn from(src: ::std::option::NoneError) -> InflateError { InflateError::UnexpectedEof }
//}
fn option_to_result<A>(a: Option<A>) -> Result<A, InflateError> {
	match a {
		Some(x) => Ok(x),
		None => Err(InflateError::UnexpectedEof),
	}
}

/// Decompress the input deflate stream
pub fn inflate<I: Iterator<Item = u8>>(input: &mut I) -> Result<Vec<u8>, InflateError> {
	let mut bitreader = Bits::new(input);
	let mut retval: Vec<u8> = Vec::new();
	let mut read_final_chunk: bool = false;

	while !read_final_chunk {
		read_final_chunk = option_to_result(bitreader.next())?;
		let typ: u16 = option_to_result(bitreader.read_n_rev(u4::_2))?;
		match typ {
			0 => {
				// no compression
				let bytes = bitreader.discard_til_byte_boundary();
				let len = u16::from_le_bytes([
					option_to_result(bytes.next())?,
					option_to_result(bytes.next())?,
				]);
				let nlen = u16::from_le_bytes([
					option_to_result(bytes.next())?,
					option_to_result(bytes.next())?,
				]);
				if len != !nlen {
					return Err(InflateError::NonCompressedLengthInvalid);
				}
				for _ in 0..len {
					retval.push(option_to_result(bytes.next())?);
				}
				bitreader = Bits::new(bytes);
			},
			1 => {
				// fixed codes
				loop {
					let code = option_to_result(decode_fixed_huffman_code(&mut bitreader))?;
					match code.cmp(&256) {
						::std::cmp::Ordering::Equal => {
							break;
						},
						::std::cmp::Ordering::Less => {
							retval.push((code & 0xFF) as u8);
						},
						::std::cmp::Ordering::Greater => {
							let length_index = code - 257;
							let length_extra_bits = LENGTH_EXTRA_BITS[usize::from(length_index)];
							let length: u16 = 3
								+ option_to_result(bitreader.read_n_rev(length_extra_bits))?
								+ LENGTH_EXTRA_BITS
									.iter()
									.take(usize::from(length_index))
									.map(|x| x.nth_bit())
									.sum::<u16>();
							let length = if code == 285 { length - 1 } else { length };

							let distance_index = option_to_result(bitreader.read_n(u4::_5))?;
							let distance_extra_bits =
								DISTANCE_EXTRA_BITS[usize::from(distance_index)];
							let distance: u16 = 1
								+ option_to_result(bitreader.read_n_rev(distance_extra_bits))?
								+ DISTANCE_EXTRA_BITS
									.iter()
									.take(usize::from(distance_index))
									.map(|x| x.nth_bit())
									.sum::<u16>();

							for _ in 0..length {
								let index_to_copy = retval.len() - usize::from(distance);
								let value_to_copy = *retval.get(index_to_copy).unwrap();
								retval.push(value_to_copy);
							}
						},
					}
				}
			},
			2 => {
				// custom codes
				let num_length_codes = 257 + option_to_result(bitreader.read_n_rev(u4::_5))?;
				let num_distance_codes = 1 + option_to_result(bitreader.read_n_rev(u4::_5))?;
				let num_meta_codes = 4 + option_to_result(bitreader.read_n_rev(u4::_4))?;

				//eprintln!("Lengths: {} {} {}", num_meta_codes, num_length_codes, num_distance_codes);
				let mut meta_code_lengths: [u4; 19] = [u4::_0; 19];
				for x in 0..usize::from(num_meta_codes) {
					meta_code_lengths[META_CODES_ORDER[x]] =
						u4::truncate(option_to_result(bitreader.read_n_rev(u4::_3))? as u8);
				}

				let meta_codes = DynamicHuffmanCodes::from_lengths(&meta_code_lengths);
				//eprintln!("{:?}", meta_codes);

				let mut length_codes: Vec<u4> = Vec::new();
				while length_codes.len() < usize::from(num_length_codes) {
					let meta = option_to_result(meta_codes.decode(&mut bitreader))?;
					act_upon_meta_code(&mut length_codes, &mut bitreader, meta)?;
				}
				let length_codes = DynamicHuffmanCodes::from_lengths(&length_codes);

				let mut distance_codes: Vec<u4> = Vec::new();
				while distance_codes.len() < usize::from(num_distance_codes) {
					let meta = option_to_result(meta_codes.decode(&mut bitreader))?;
					act_upon_meta_code(&mut distance_codes, &mut bitreader, meta)?;
				}
				let distance_codes = DynamicHuffmanCodes::from_lengths(&distance_codes);

				loop {
					let code = option_to_result(length_codes.decode(&mut bitreader))?;
					match code.cmp(&256) {
						::std::cmp::Ordering::Equal => break,
						::std::cmp::Ordering::Less => {
							retval.push((code & 0xFF) as u8);
						},
						::std::cmp::Ordering::Greater => {
							let length_index = code - 257;
							let length_extra_bits = LENGTH_EXTRA_BITS[usize::from(length_index)];
							let length: u16 = 3
								+ option_to_result(bitreader.read_n_rev(length_extra_bits))?
								+ LENGTH_EXTRA_BITS
									.iter()
									.take(usize::from(length_index))
									.map(|x| x.nth_bit())
									.sum::<u16>();
							let length = if code == 285 { length - 1 } else { length };

							let distance_index =
								option_to_result(distance_codes.decode(&mut bitreader))?;
							let distance_extra_bits =
								DISTANCE_EXTRA_BITS[usize::from(distance_index)];
							let distance: u16 = 1
								+ option_to_result(bitreader.read_n_rev(distance_extra_bits))?
								+ DISTANCE_EXTRA_BITS
									.iter()
									.take(usize::from(distance_index))
									.map(|x| x.nth_bit())
									.sum::<u16>();

							for _ in 0..length {
								let index_to_copy = retval.len() - usize::from(distance);
								let value_to_copy = *retval.get(index_to_copy).unwrap();
								retval.push(value_to_copy);
							}
						},
					}
				}
			},
			_ => {
				// error
				return Err(InflateError::InvalidBtype);
			},
		}
	}
	Ok(retval)
}

/// Store the the input in a deflate stream entirely using immediate mode (00)
pub fn deflate_immediate<I: Iterator<Item = u8>>(input: I) -> Vec<u8> {
	let input: Vec<u8> = input.collect();
	let input = input.chunks(0xFFFF);

	let last_item = input.len() - 1;
	let input = input.zip(0..).map(|(chunk, idx)| (chunk, idx == last_item));
	// I'd love to use an Iterator::flat_map here, but I can't figure out how to get the slices in the flat_map to live long enough
	let mut retval: Vec<u8> = Vec::new();
	for (chunk, is_last) in input {
		retval.push(u8::from(is_last));
		retval.extend((chunk.len() as u16).to_le_bytes().iter());
		retval.extend((!(chunk.len() as u16)).to_le_bytes().iter());
		retval.extend(chunk.iter());
	}
	retval
}

/// Decode a single a fixed-mode huffman code from the given stream
fn decode_fixed_huffman_code<I: Iterator<Item = u8>>(bitreader: &mut Bits<I>) -> Option<u16> {
	match bitreader.read_n(u4::_2)? {
		0 => match bitreader.read_n(u4::_2)? {
			3 => bitreader.read_n(u4::_4),
			x => bitreader.read_n(u4::_3).map(|y| 256 + (x << 3) + y),
		},
		1 => bitreader.read_n(u4::_6).map(|y| 16 + y),
		2 => bitreader.read_n(u4::_6).map(|y| 80 + y),
		3 => match bitreader.read_n(u4::_3)? {
			0 => bitreader.read_n(u4::_3).map(|y| 280 + y),
			x => bitreader.read_n(u4::_4).map(|y| 144 + ((x - 1) << 4) + y),
		},
		_ => panic!(""),
	}
}

#[derive(Debug)]
struct DynamicHuffmanCodeValue {
	length: u4,
	value: u16,
}

/// Huffman codes that are encoded in the PNG stream
#[derive(Debug)]
struct DynamicHuffmanCodes {
	backing: Vec<DynamicHuffmanCodeValue>,
}

impl DynamicHuffmanCodes {
	/// Create the set of codes from the code lengths of each item
	fn from_lengths(lengths: &[u4]) -> DynamicHuffmanCodes {
		let mut lengths: Vec<DynamicHuffmanCodeValue> = lengths
			.iter()
			.cloned()
			.zip(0..)
			.map(|(length, value)| DynamicHuffmanCodeValue { length, value })
			.filter(|x| x.length != u4::_0)
			.collect();
		lengths.sort_by_key(|x| x.length);
		DynamicHuffmanCodes { backing: lengths }
	}

	/// Decodes a value using this set of dynamic huffman codes
	fn decode<I: Iterator<Item = u8>>(&self, bitreader: &mut Bits<I>) -> Option<u16> {
		let mut index: usize = 0;
		let mut index_code: u16 = 0;
		let mut read_code: u16 = 0;
		let mut read_len: u4 = u4::_0;

		//eprintln!("Enter");
		//eprintln!("  self: {:?}", self);
		loop {
			//eprintln!("  Start of loop");
			while read_len < self.backing[index].length {
				read_code += if bitreader.next()? {
					read_len.nth_bit()
				} else {
					0
				};
				read_len += u4::_1;
				//eprintln!("    Read Code: {:0w$b}", read_code, w = usize::from(read_len));
			}
			if read_len == self.backing[index].length && read_code == index_code {
				//eprintln!("    Ret Code: {:0w$b}", read_code, w = usize::from(read_len));
				//eprintln!("    Retval: {}", self.backing[index].value);
				break Some(self.backing[index].value);
			}
			index_code = u16_reverse_bits(index_code);
			index_code += (u4::_F - self.backing[index].length + u4::_1).nth_bit();
			index_code = u16_reverse_bits(index_code);
			index += 1;
			if index >= self.backing.len() {
				panic!("Code not found");
			}
			//eprintln!("    Index Code: {:0w$b}", index_code, w = usize::from(self.backing[index].length));
		}
	}
}

/// Appends values to the results vector based on the provided meta code, and possibly the next few bits in the bitreader.
fn act_upon_meta_code<I: Iterator<Item = u8>>(
	results: &mut Vec<u4>,
	bitreader: &mut Bits<I>,
	code: u16,
) -> Result<(), InflateError> {
	if code < 16 {
		results.push(u4::truncate(code as u8));
	} else if code == 16 {
		let prev_code = results[results.len() - 1];
		let times = 3 + option_to_result(bitreader.read_n_rev(u4::_2))?;
		for _ in 0..times {
			results.push(prev_code);
		}
	} else if code == 17 {
		let times = 3 + option_to_result(bitreader.read_n_rev(u4::_3))?;
		for _ in 0..times {
			results.push(u4::_0);
		}
	} else if code == 18 {
		let times = 11 + option_to_result(bitreader.read_n_rev(u4::_7))?;
		for _ in 0..times {
			results.push(u4::_0);
		}
	} else {
		panic!("Illegal meta code: {}", code);
	}
	Ok(())
}

/// "u16::reverse_bits is an experimental API"
fn u16_reverse_bits(a: u16) -> u16 {
	let mut retval = 0;
	for bit in 0..16 {
		retval += if (a & (1 << bit)) != 0 {
			1 << (15 - bit)
		} else {
			0
		}
	}
	retval
}

// bits are intentionally grouped by huffman instruction, not by nibble
#[allow(clippy::unusual_byte_groupings)]
#[cfg(test)]
mod tests {
	mod decode_fixed_huffman_code {
		use super::super::decode_fixed_huffman_code;

		#[test]
		fn hit_every_output() {
			let mut res: [bool; 288] = [false; 288];

			for i in u16::MIN..u16::MAX {
				let bits: [u8; 2] = [((i >> 8) & 0xFF) as u8, (i & 0xFF) as u8];
				let bits = bits.iter().cloned();
				let mut bits = super::super::super::bits::Bits::new(bits);
				res[usize::from(decode_fixed_huffman_code(&mut bits).unwrap())] = true;
			}

			for (index, result_item_was_hit) in res.iter().enumerate() {
				assert!(result_item_was_hit, "i = {}", index);
			}
		}
	}

	mod dynamic_huffman_codes {
		use super::super::super::u4;
		use super::super::super::Bits;
		use super::super::DynamicHuffmanCodes;

		fn assert_decode(expected: u16, dut: &DynamicHuffmanCodes, huffman_code: u8) {
			assert_eq!(
				expected,
				dut.decode(&mut Bits::new([huffman_code].iter().cloned()))
					.unwrap()
			);
		}

		#[test]
		fn two_constant_length() {
			let dut = [u4::_1, u4::_1];
			let dut = DynamicHuffmanCodes::from_lengths(&dut);

			assert_decode(0, &dut, 0b0u8);
			assert_decode(1, &dut, 0b1u8);
		}
		#[test]
		fn four_constant_length() {
			let dut = [u4::_2, u4::_2, u4::_2, u4::_2];
			let dut = DynamicHuffmanCodes::from_lengths(&dut);

			assert_decode(0, &dut, 0b00u8);
			assert_decode(1, &dut, 0b10u8);
			assert_decode(2, &dut, 0b01u8);
			assert_decode(3, &dut, 0b11u8);
		}
		#[test]
		fn four_constant_delta() {
			let dut = [u4::_1, u4::_2, u4::_3, u4::_4];
			let dut = DynamicHuffmanCodes::from_lengths(&dut);

			assert_decode(0, &dut, 0b000_0u8);
			assert_decode(1, &dut, 0b00_01u8);
			assert_decode(2, &dut, 0b0_011u8);
			assert_decode(3, &dut, 0b_0111u8);
		}
		#[test]
		fn provided_sample() {
			#[rustfmt::skip]
			let dut = [u4::_3, u4::_3, u4::_3, u4::_3, u4::_3, u4::_2, u4::_4, u4::_4];
			let dut = DynamicHuffmanCodes::from_lengths(&dut);

			assert_decode(5, &dut, 0b00_00u8);
			assert_decode(0, &dut, 0b0_010u8);
			assert_decode(1, &dut, 0b0_110u8);
			assert_decode(2, &dut, 0b0_001u8);
			assert_decode(3, &dut, 0b0_101u8);
			assert_decode(4, &dut, 0b0_011u8);
			assert_decode(6, &dut, 0b_0111u8);
			assert_decode(7, &dut, 0b_1111u8);
		}
	}

	mod inflate_samples {
		use super::super::inflate;
		#[test]
		fn immediate_mode() {
			#[rustfmt::skip]
			let exp: [u8; 10] = [1,2,3,4,5,6,7,8,9,10];
			#[rustfmt::skip]
			let dut: [u8; 15] = [1, 10, 0, !10, 0xFF, 1,2,3,4,5,6,7,8,9,10];
			let mut dut = dut.iter().cloned();
			let res = inflate(&mut dut).unwrap();
			assert!(exp == res.as_slice(), "{:?}", res);
		}
		#[test]
		fn one_zero() {
			let exp: [u8; 1] = [0];
			let dut: [u8; 3] = [0x63, 0x00, 0x00];
			let mut dut = dut.iter().cloned();
			let res = inflate(&mut dut).unwrap();
			assert!(exp == res.as_slice(), "{:?}", res);
		}
		#[test]
		fn four_zero() {
			let exp: [u8; 4] = [0; 4];
			let dut: [u8; 4] = [0x63, 0x00, 0x02, 0x00];
			let mut dut = dut.iter().cloned();
			let res = inflate(&mut dut).unwrap();
			assert!(exp == res.as_slice(), "{:?}", res);
		}
		#[test]
		fn abcde_times_five() {
			#[rustfmt::skip]
			let exp: [u8; 25] = [0x61, 0x62, 0x63, 0x64, 0x65, 0x61, 0x62, 0x63, 0x64, 0x65, 0x61, 0x62, 0x63, 0x64, 0x65, 0x61, 0x62, 0x63, 0x64, 0x65, 0x61, 0x62, 0x63, 0x64, 0x65];
			#[rustfmt::skip]
			let dut: [u8; 9] = [0x4b, 0x4c, 0x4a, 0x4e, 0x49, 0xc5, 0x46, 0x00, 0x00];
			let mut dut = dut.iter().cloned();
			let res = inflate(&mut dut).unwrap();
			assert!(exp == res.as_slice(), "{:?}", res);
		}
		#[test]
		fn black_square_4x4() {
			#[rustfmt::skip]
			let exp: [u8; 68] = [
				0, 0,0,0,255, 0,0,0,255, 0,0,0,255, 0,0,0,255,
				0, 0,0,0,255, 0,0,0,255, 0,0,0,255, 0,0,0,255,
				0, 0,0,0,255, 0,0,0,255, 0,0,0,255, 0,0,0,255,
				0, 0,0,0,255, 0,0,0,255, 0,0,0,255, 0,0,0,255,
			];
			#[rustfmt::skip]
			let dut: [u8; 21] = [0x9d, 0xc8, 0xb1, 0x0d, 0x00, 0x00, 0x00, 0x82, 0x30, 0xff, 0x7f, 0x5a, 0x1d, 0x99, 0x21, 0x61, 0x69, 0x5e, 0xb9, 0x80, 0x01];
			let mut dut = dut.iter().cloned();
			let res = inflate(&mut dut).unwrap();
			assert_eq!(exp.len(), res.len(), "LENGTH");
			assert!(exp.iter().zip(res.iter()).all(|(a, b)| a == b));
		}
	}

	mod inflate_instructions {
		use super::super::inflate;

		#[test]
		fn immediate() {
			let exp: Vec<u8> = (0..=255).collect();
			let dut: Vec<u8> = [1, 0, 1, 0xFF, 0xFE]
				.iter()
				.cloned()
				.chain(0..=255)
				.collect();
			let mut dut = dut.iter().cloned();
			let res = inflate(&mut dut).unwrap();
			assert_eq!(exp.len(), res.len(), "LENGTH");
			assert!(exp.iter().zip(res.iter()).all(|(a, b)| a == b));
		}

		#[test]
		fn fixed_coded_l003d1() {
			let exp: Vec<u8> = (0..=255).chain([255, 255, 255].iter().cloned()).collect();
			let dut: Vec<u8> = [0, 0, 1, 0xFF, 0xFE]
				.iter()
				.cloned()
				.chain(0..=255)
				.chain([0b00000_011, 0b0_00000_10, 0b000000].iter().cloned())
				.collect();
			let mut dut = dut.iter().cloned();
			let res = inflate(&mut dut).unwrap();
			assert_eq!(exp.len(), res.len(), "LENGTH");
			assert!(exp.iter().zip(res.iter()).all(|(a, b)| a == b));
		}

		#[test]
		fn fixed_coded_l010d1() {
			let exp: Vec<u8> = (0..=255).chain(std::iter::repeat(255).take(10)).collect();
			let dut: Vec<u8> = [0, 0, 1, 0xFF, 0xFE]
				.iter()
				.cloned()
				.chain(0..=255)
				.chain([0b01000_011, 0b0_00000_00, 0b000000].iter().cloned())
				.collect();
			let mut dut = dut.iter().cloned();
			let res = inflate(&mut dut).unwrap();
			assert_eq!(exp.len(), res.len(), "LENGTH");
			assert!(exp.iter().zip(res.iter()).all(|(a, b)| a == b));
		}

		#[test]
		fn fixed_coded_l257d1() {
			let exp: Vec<u8> = (0..=255).chain(std::iter::repeat(255).take(257)).collect();
			let dut: Vec<u8> = [0, 0, 1, 0xFF, 0xFE]
				.iter()
				.cloned()
				.chain(0..=255)
				.chain([0b00011_011, 0b_11110_001, 0, 0].iter().cloned())
				.collect();
			let mut dut = dut.iter().cloned();
			let res = inflate(&mut dut).unwrap();
			assert_eq!(exp.len(), res.len(), "LENGTH");
			assert!(exp.iter().zip(res.iter()).all(|(a, b)| a == b));
		}

		#[test]
		fn fixed_coded_l258d1() {
			let exp: Vec<u8> = (0..=255).chain(std::iter::repeat(255).take(258)).collect();
			let dut: Vec<u8> = [0, 0, 1, 0xFF, 0xFE]
				.iter()
				.cloned()
				.chain(0..=255)
				.chain([0b00011_011, 0b_00000_101, 0].iter().cloned())
				.collect();
			let mut dut = dut.iter().cloned();
			let res = inflate(&mut dut).unwrap();
			assert_eq!(exp.len(), res.len(), "LENGTH");
			assert!(exp.iter().zip(res.iter()).all(|(a, b)| a == b));
		}

		#[test]
		fn fixed_coded_l003d4() {
			let exp: Vec<u8> = (0..=255).chain([252, 253, 254].iter().cloned()).collect();
			let dut: Vec<u8> = [0, 0, 1, 0xFF, 0xFE]
				.iter()
				.cloned()
				.chain(0..=255)
				.chain([0b00000_011, 0b0_11000_10, 0b000000].iter().cloned())
				.collect();
			let mut dut = dut.iter().cloned();
			let res = inflate(&mut dut).unwrap();
			assert_eq!(exp.len(), res.len(), "LENGTH");
			assert!(exp.iter().zip(res.iter()).all(|(a, b)| a == b));
		}

		#[test]
		fn fixed_coded_l003d97() {
			let exp: Vec<u8> = (0..=255).chain([159, 160, 161].iter().cloned()).collect();
			let dut: Vec<u8> = [0, 0, 1, 0xFF, 0xFE]
				.iter()
				.cloned()
				.chain(0..=255)
				.chain([0b00000_011, 0b0_10110_10, 0, 0].iter().cloned())
				.collect();
			let mut dut = dut.iter().cloned();
			let res = inflate(&mut dut).unwrap();
			assert_eq!(exp.len(), res.len(), "LENGTH");
			assert!(exp.iter().zip(res.iter()).all(|(a, b)| a == b));
		}

		#[test]
		fn fixed_coded_l003d128() {
			let exp: Vec<u8> = (0..=255).chain([128, 129, 130].iter().cloned()).collect();
			let dut: Vec<u8> = [0, 0, 1, 0xFF, 0xFE]
				.iter()
				.cloned()
				.chain(0..=255)
				.chain([0b00000_011, 0b1_10110_10, 0b0000_1111, 0].iter().cloned())
				.collect();
			let mut dut = dut.iter().cloned();
			let res = inflate(&mut dut).unwrap();
			assert_eq!(exp.len(), res.len(), "LENGTH");
			assert!(exp.iter().zip(res.iter()).all(|(a, b)| a == b));
		}

		#[test]
		fn fixed_codes_l003d24577() {
			let base: Vec<u8> = std::iter::repeat(5)
				.take(16)
				.chain(std::iter::repeat(80).take(3))
				.chain(std::iter::repeat(120).take(24574))
				.collect();
			let exp: Vec<u8> = base
				.iter()
				.cloned()
				.chain(std::iter::repeat(80).take(3))
				.collect();
			let dut: Vec<u8> = [0, 0x11, 0x60, 0xEE, 0x9F]
				.iter()
				.cloned()
				.chain(base)
				.chain([0b00000_011, 0b0_10111_10, 0, 0, 0].iter().cloned())
				.collect();
			let mut dut = dut.iter().cloned();
			let res = inflate(&mut dut).unwrap();
			assert_eq!(exp.len(), res.len(), "LENGTH");
			assert!(exp.iter().zip(res.iter()).all(|(a, b)| a == b));
		}

		#[test]
		fn fixed_codes_l003d32768() {
			let base: Vec<u8> = std::iter::repeat(5)
				.take(16)
				.chain(std::iter::repeat(80).take(3))
				.chain(std::iter::repeat(120).take(32768 - 3))
				.collect();
			let exp: Vec<u8> = base
				.iter()
				.cloned()
				.chain(std::iter::repeat(80).take(3))
				.collect();
			let dut: Vec<u8> = [0, 0x10, 0x80, 0xEF, 0x7F]
				.iter()
				.cloned()
				.chain(base)
				.chain(
					[0b00000_011, 0b1_10111_10, 0b11111111, 0b1111, 0]
						.iter()
						.cloned(),
				)
				.collect();
			let mut dut = dut.iter().cloned();
			let res = inflate(&mut dut).unwrap();
			assert_eq!(exp.len(), res.len(), "LENGTH");
			assert!(exp.iter().zip(res.iter()).all(|(a, b)| a == b));
		}
	}
}
