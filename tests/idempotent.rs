//! For each valid test case, asserts that the dut is idempotent

extern crate png_inflate_derive;
extern crate tempfile;

use png_inflate_derive::generate_for_each_files;
use std::fs::read;
use std::path::Path;
use std::process::Command;
use tempfile::NamedTempFile;

const PROGRAM_EXE: &str = env!("CARGO_BIN_EXE_png_inflate");

generate_for_each_files!();

fn test_one(infile: &Path, extra_args: &[&str]) {
	let out1file = NamedTempFile::new().expect("").into_temp_path();
	let out2file = NamedTempFile::new().expect("").into_temp_path();

	let output1 = Command::new(PROGRAM_EXE)
		.arg(infile)
		.arg(&out1file)
		.args(extra_args)
		.output()
		.expect("failed to execute first subprocess");
	assert!(
		output1.status.success(),
		"first subprocess execution was not success\n\n-- stderr1:\n{}\n",
		std::str::from_utf8(&output1.stderr).expect("")
	);
	let output2 = Command::new(PROGRAM_EXE)
		.arg(&out1file)
		.arg(&out2file)
		.args(extra_args)
		.output()
		.expect("failed to execute second process");
	assert!(
		output2.status.success(),
		"second subprocess execution was not success\n\n-- stderr1:\n{}\n\n-- stderr2:\n{}\n",
		std::str::from_utf8(&output1.stderr).expect(""),
		std::str::from_utf8(&output2.stderr).expect("")
	);

	let res1 = read(out1file).expect("could not read first subprocess's output file");
	let res2 = read(out2file).expect("could not read second subprocess's output file");
	assert_eq!(res1, res2);
}

mod noargs {
	for_each_valid_file!(super::test_one, &[]);
	for_each_otherinvalid_file!(super::test_one, &[]);
}
mod copy_unsafe {
	for_each_unsafecopy_file!(super::test_one, &["--copy-unsafe"]);
}
