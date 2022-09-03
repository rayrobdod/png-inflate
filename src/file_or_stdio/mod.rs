//! ::std:io::{Read, Write} implementations that are a sum type of other
//! ::std::io::{Read, Write} implementations

extern crate atomicwrites;
use std::io::Write;

#[derive(Debug)]
pub enum FileOrStdin {
	File(::std::fs::File),
	Stdin(::std::io::Stdin),
}

impl From<Option<String>> for FileOrStdin {
	fn from(src: Option<String>) -> FileOrStdin {
		match src {
			None => FileOrStdin::Stdin(::std::io::stdin()),
			Some(s) => FileOrStdin::File(
				::std::fs::OpenOptions::new()
					.read(true)
					.open(s)
					.expect("Could not open input file"),
			),
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

pub enum FileOrStdout {
	File(atomicwrites::AtomicFile),
	Stdout(::std::io::Stdout),
}

impl From<Option<String>> for FileOrStdout {
	fn from(src: Option<String>) -> FileOrStdout {
		match src {
			None => FileOrStdout::Stdout(::std::io::stdout()),
			Some(s) => FileOrStdout::File(atomicwrites::AtomicFile::new(
				s,
				atomicwrites::AllowOverwrite,
			)),
		}
	}
}

impl FileOrStdout {
	pub fn write<R, F>(&mut self, f: F) -> ::std::io::Result<R>
	where
		F: FnOnce(&mut dyn Write) -> ::std::io::Result<R>,
	{
		match self {
			FileOrStdout::File(x) => {
				let a: std::result::Result<R, atomicwrites::Error<std::io::Error>> =
					x.write(|file| f(file));
				let b: std::result::Result<R, std::io::Error> = a.map_err(|e| e.into());
				b
			},
			FileOrStdout::Stdout(x) => f(x),
		}
	}
}
