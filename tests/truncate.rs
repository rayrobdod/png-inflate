///! Asserts that, when the dut writes to an output file, that output file is truncated

extern crate tempfile;
extern crate png_inflate_derive;

use ::std::path::Path;
use ::std::process::Command;
use tempfile::NamedTempFile;
use png_inflate_derive::generate_for_each_files;

const PROGRAM_EXE:&str = env!("CARGO_BIN_EXE_png_inflate");

generate_for_each_files!();

fn test_one(infile:&Path) {
	let outfile = NamedTempFile::new().expect("");

	let initial_size = if Some(::std::ffi::OsStr::new("PngSuite.png")) == infile.file_name() {200000} else {12 * 1024};
	outfile.as_file().set_len(initial_size).expect("failed to initialize output file");

	let outfile = outfile.into_temp_path();

	let output = Command::new(PROGRAM_EXE)
		.arg(&infile)
		.arg(&outfile)
		.output()
		.expect("failed to execute subprocess");
	assert!(output.status.success(), "subprocess execution was not success\n\n-- stderr:\n{}\n", std::str::from_utf8(&output.stderr).expect(""));

	assert!(outfile.metadata().expect("").len() < initial_size, "outfile was not shrunk: {}", outfile.metadata().expect("").len());
}

for_each_valid_file!(test_one);
for_each_otherinvalid_file!(test_one);
