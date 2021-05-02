///! Checks that files that should be rejected are rejected

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
	let outfile = outfile.into_temp_path();

	let output = Command::new(PROGRAM_EXE)
		.arg(&infile)
		.arg(&outfile)
		.output()
		.expect("failed to execute subprocess");
	assert!(!output.status.success(), "subprocess execution should not have been success\n\n-- stderr:\n{}\n", std::str::from_utf8(&output.stderr).expect(""));
	// TODO: check the message?
	assert!(outfile.metadata().expect("").len() == 0, "outfile was written to: {}", outfile.metadata().expect("").len());
}

for_each_badmagic_file!(test_one);
for_each_badchecksum_file!(test_one);
