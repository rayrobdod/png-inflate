//! Contains an iterator transformer that transforms a iterator\<u8\> iterator over the bits of that stream
use super::u4;
use super::u4ZeroToRangeIter;

/// An iterator transformer that splits each u8 into its component bits,
/// reading the bytes in LSB order
pub struct Bits<I: Iterator<Item = u8>> {
	/// the source of bytes
	backing: I,
	/// the current byte being read from
	current_byte: u8,
	/// the number of bits that have been read from the current byte
	current_byte_read_bits: u8,
}

impl<I: Iterator<Item = u8>> Iterator for Bits<I> {
	type Item = bool;

	fn next(&mut self) -> Option<bool> {
		if self.current_byte_read_bits >= 8 {
			self.current_byte_read_bits = 0;
			match self.backing.next() {
				Some(x) => self.current_byte = x,
				None => return None,
			}
		}

		let retval = (self.current_byte & 0x01) != 0;
		self.current_byte >>= 1;
		self.current_byte_read_bits += 1;
		Some(retval)
	}
}

impl<I: Iterator<Item = u8>> Bits<I> {
	pub fn new(backing: I) -> Bits<I> {
		Bits {
			backing: backing,
			current_byte: 0,
			current_byte_read_bits: u8::max_value(),
		}
	}

	/// reads n bits from this Iterator, packing the result into a single u16,
	/// such that the first bit read becomes the MSB of the returned value
	pub fn read_n(&mut self, bit_count: u4) -> Option<u16> {
		let mut retval: u16 = 0;
		for _ in u4ZeroToRangeIter::new(bit_count) {
			retval <<= 1;
			match self.next() {
				Some(x) => retval += u16::from(x),
				None => return None,
			}
		}
		Some(retval)
	}

	/// reads n bits from this Iterator, packing the result into a single u16
	/// in the reverse order of `read_n`
	pub fn read_n_rev(&mut self, bit_count: u4) -> Option<u16> {
		let mut retval: u16 = 0;
		for i in u4ZeroToRangeIter::new(bit_count) {
			match self.next() {
				Some(x) => retval += u16::from(x) * i.nth_bit(),
				None => return None,
			}
		}
		Some(retval)
	}

	/// Discards any bits remaining in the current byte
	pub fn discard_til_byte_boundary(self) -> I {
		self.backing
	}
}

#[cfg(test)]
mod tests {
	mod read_1 {
		use super::super::Bits;

		#[test]
		fn one_byte() {
			let dut: [u8; 1] = [0b11010110];
			let dut = dut.iter().cloned();
			let mut dut = Bits::new(dut);

			assert!(false == dut.next().unwrap());
			assert!(true == dut.next().unwrap());
			assert!(true == dut.next().unwrap());
			assert!(false == dut.next().unwrap());
			assert!(true == dut.next().unwrap());
			assert!(false == dut.next().unwrap());
			assert!(true == dut.next().unwrap());
			assert!(true == dut.next().unwrap());
			assert!(dut.next().is_none());
		}
	}
	mod read_n {
		use super::super::super::u4;
		use super::super::Bits;

		#[test]
		fn one_byte() {
			let dut: [u8; 1] = [0b11010110];
			let dut = dut.iter().cloned();
			let mut dut = Bits::new(dut);

			assert!(0b0110 == dut.read_n(u4::_4).unwrap());
			assert!(0b1011 == dut.read_n(u4::_4).unwrap());
			assert!(dut.next().is_none());
		}

		#[test]
		fn one_byte_rev() {
			let dut: [u8; 1] = [0b11010110];
			let dut = dut.iter().cloned();
			let mut dut = Bits::new(dut);

			assert!(0b0110 == dut.read_n_rev(u4::_4).unwrap());
			assert!(0b1101 == dut.read_n_rev(u4::_4).unwrap());
			assert!(dut.next().is_none());
		}
	}
}
