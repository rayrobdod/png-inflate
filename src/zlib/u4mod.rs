///! Contains a numeric type that holds four bits of information
//const b0000u4 = u4::_0;
//const b0001u4 = u4::_1;
//const b0010u4 = u4::_2;
//....
//const b1111u4 = u4::_F;
//const x0u4 = u4::_0;
//const x1u4 = u4::_1;
//....
//const xFu4 = u4::_F;

#[allow(non_camel_case_types)]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
/// Represents a four-bit number
pub enum u4 {
	_0, _1, _2, _3, _4, _5, _6, _7,
	_8, _9, _A, _B, _C, _D, _E, _F,
}

impl u4 {
	pub fn truncate(src:u8) -> u4 {
		match src & 0xF {
			0 => u4::_0, 1 => u4::_1, 2 => u4::_2, 3 => u4::_3,
			4 => u4::_4, 5 => u4::_5, 6 => u4::_6, 7 => u4::_7,
			8 => u4::_8, 9 => u4::_9, 10 => u4::_A, 11 => u4::_B,
			12 => u4::_C, 13 => u4::_D, 14 => u4::_E, 15 => u4::_F,
			_ => panic!("value & 0xF was not between 0 and 15: {}", src & 0xF)
		}
	}

	/// Returns a u16 with the value `2 ^^ self`
	pub fn nth_bit(self) -> u16 {
		match self {
			u4::_0 => 0x0001, u4::_1 => 0x0002, u4::_2 => 0x0004, u4::_3 => 0x0008,
			u4::_4 => 0x0010, u4::_5 => 0x0020, u4::_6 => 0x0040, u4::_7 => 0x0080,
			u4::_8 => 0x0100, u4::_9 => 0x0200, u4::_A => 0x0400, u4::_B => 0x0800,
			u4::_C => 0x1000, u4::_D => 0x2000, u4::_E => 0x4000, u4::_F => 0x8000,
		}
	}

	// A reasonable language would let me `impl From<u8> for (u4, u4)`. Rust does not.
	pub fn split(src:u8) -> (u4, u4) {
		( u4::truncate(src >> 4), u4::truncate(src) )
	}

	pub fn concat(a:u4, b:u4) -> u8 {
		u8::from(a) << 4 | u8::from(b)
	}
}

impl From<u4> for u8 {
	fn from(src:u4) -> u8 {
		/*
		match src {
			u4::_0 => 0, u4::_1 => 1, u4::_2 => 2, u4::_3 => 3,
			u4::_4 => 4, u4::_5 => 5, u4::_6 => 6, u4::_7 => 7,
			u4::_8 => 8, u4::_9 => 9, u4::_A => 10, u4::_B => 11,
			u4::_C => 12, u4::_D => 13, u4::_E => 14, u4::_F => 15,
		}
		*/
		src as u8
	}
}

impl From<u4> for usize { fn from(src:u4) -> usize { usize::from(u8::from(src)) } }

impl ::std::ops::AddAssign for u4 {
	fn add_assign(&mut self, other:u4) -> () {
		*self = *self + other;
	}
}

impl ::std::ops::Add for u4 {
	type Output = u4;
	#[cfg(debug_assertions)]
	fn add(self, other:u4) -> u4 {
		let (chk, ret) = u4::split(u8::from(self) + u8::from(other));
		if chk != u4::_0 {
			panic!("Overflow in u4::add -- {:?} + {:?}", self, other)
		} else {
			ret
		}
	}
	#[cfg(not(debug_assertions))]
	fn add(self, other:u4) -> u4 {
		u4::truncate(u8::from(self) + u8::from(other))
	}
}

impl ::std::ops::Sub for u4 {
	type Output = u4;
	#[cfg(debug_assertions)]
	fn sub(self, other:u4) -> u4 {
		let (chk, ret) = u4::split(u8::from(self) - u8::from(other));
		if chk != u4::_0 {
			panic!("Overflow in u4::sub -- {:?} - {:?}", self, other)
		} else {
			ret
		}
	}
	#[cfg(not(debug_assertions))]
	fn sub(self, other:u4) -> u4 {
		u4::truncate(u8::from(self) - u8::from(other))
	}
}

pub struct ZeroToRangeIter {
	current:u4,
	end:u4,
}

impl ZeroToRangeIter {
	pub fn new(end:u4) -> ZeroToRangeIter {
		ZeroToRangeIter { current : u4::_0, end : end }
	}
}

impl Iterator for ZeroToRangeIter {
	type Item = u4;
	fn next(&mut self) -> Option<u4> {
		if self.current >= self.end {
			None
		} else {
			let retval = self.current;
			self.current = self.current + u4::_1;
			Some(retval)
		}
	}
}
