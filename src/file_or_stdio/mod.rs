///! ::std:io::{Read, Write} implementations that are a sum type of other
///! ::std::io::{Read, Write} implementations

#[derive(Debug)]
pub enum FileOrStdin {
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
pub enum FileOrStdout {
	File(::std::fs::File),
	Stdout(::std::io::Stdout),
}

impl From<Option<String>> for FileOrStdout {
	fn from(src:Option<String>) -> FileOrStdout {
		match src {
			None => FileOrStdout::Stdout(::std::io::stdout()),
			Some(s) => FileOrStdout::File(::std::fs::OpenOptions::new().write(true).create(true).open(s).expect("Could not open output file")),
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
