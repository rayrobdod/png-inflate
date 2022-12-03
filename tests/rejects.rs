//! Checks that files that should be rejected are rejected

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
	let outfile = outfile.into_temp_path();

	let output = Command::new(PROGRAM_EXE)
		.arg(infile)
		.arg(&outfile)
		.args(extra_args)
		.output()
		.expect("failed to execute subprocess");
	assert!(
		!output.status.success(),
		"subprocess execution should not have been success\n\n-- stderr:\n{}\n",
		std::str::from_utf8(&output.stderr).expect("")
	);
	// TODO: check the message?
	assert!(
		outfile.metadata().expect("").len() == 0,
		"outfile was written to: {}",
		outfile.metadata().expect("").len()
	);
}

mod noargs {
	for_each_badmagic_file!(super::test_one, &[]);
	for_each_badchecksum_file!(super::test_one, &[]);

	// acTL, fcTL and fdAT are private and not safe-to-copy, and so should be
	// rejected without an argument explicitly allowing it, either `--copy-unsafe`
	// or a future `--apng` argument
	for_each_unsafecopy_file!(super::test_one, &[]);
}
