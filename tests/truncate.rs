//! Asserts that, when the dut writes to an output file, that output file is truncated

extern crate png_inflate_derive;
extern crate tempfile;

use png_inflate_derive::generate_for_each_files;
use std::path::Path;
use std::process::Command;
use tempfile::NamedTempFile;

const PROGRAM_EXE: &str = env!("CARGO_BIN_EXE_png_inflate");

generate_for_each_files!();

fn test_one(infile: &Path, extra_args: &[&str]) {
	let outfile = NamedTempFile::new().expect("");

	let initial_size = if Some(::std::ffi::OsStr::new("PngSuite.png")) == infile.file_name() {
		200000
	} else {
		12 * 1024
	};
	outfile
		.as_file()
		.set_len(initial_size)
		.expect("failed to initialize output file");

	let outfile = outfile.into_temp_path();

	let output = Command::new(PROGRAM_EXE)
		.arg(infile)
		.arg(&outfile)
		.args(extra_args)
		.output()
		.expect("failed to execute subprocess");
	assert!(
		output.status.success(),
		"subprocess execution was not success\n\n-- stderr:\n{}\n",
		std::str::from_utf8(&output.stderr).expect("")
	);

	assert!(
		outfile.metadata().expect("").len() < initial_size,
		"outfile was not shrunk: {}",
		outfile.metadata().expect("").len()
	);
}

mod noargs {
	for_each_valid_file!(super::test_one, &[]);
	for_each_otherinvalid_file!(super::test_one, &[]);
}
mod copy_unsafe {
	for_each_unsafecopy_file!(super::test_one, &["--copy-unsafe"]);
}
mod apng {
	for_each_valid_file!(super::test_one, &["--apng"]);
	for_each_apng_file!(super::test_one, &["--apng"]);
}
