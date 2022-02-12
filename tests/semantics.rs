//! For each valid test case, asserts that the dut creates semantically-identical files

extern crate tempfile;
extern crate png_inflate_derive;

use ::std::fs::read;
use ::std::path::Path;
use ::std::process::Command;
use tempfile::NamedTempFile;
use png_inflate_derive::generate_for_each_files;

const PROGRAM_EXE:&str = env!("CARGO_BIN_EXE_png_inflate");
const SNG_EXE:&str = env!("SNG");

generate_for_each_files!();

fn test_one(infile:&Path, extra_args: &[&str]) {
	let orig = NamedTempFile::new().expect("");
	let orig = orig.path();
	let orig_png = orig.with_extension("png");
	let orig_sng = orig.with_extension("sng");
	let clean = NamedTempFile::new().expect("");
	let clean = clean.path();
	let clean_png = clean.with_extension("png");
	let clean_sng = clean.with_extension("sng");

	::std::fs::copy(&infile, &orig_png).expect("Could not copy input to temp file");

	let output_sng_orig = Command::new(SNG_EXE)
		.arg(&orig_png)
		.output()
		.expect("failed to execute sng on original");
	assert!(output_sng_orig.status.success(), "sng on original execution was not success\n\n-- stderr:\n{}\n", std::str::from_utf8(&output_sng_orig.stderr).expect(""));
	let output_inflate = Command::new(PROGRAM_EXE)
		.arg(&orig_png)
		.arg(&clean_png)
		.args(extra_args)
		.output()
		.expect("failed to execute png_inflate process");
	assert!(output_inflate.status.success(), "png_inflate execution was not success\n\n-- stderr:\n{}\n", std::str::from_utf8(&output_inflate.stderr).expect(""));
	let output_sng_clean = Command::new(SNG_EXE)
		.arg(&clean_png)
		.output()
		.expect("failed to execute sng on clean");
	assert!(output_sng_clean.status.success(), "sng on original clean was not success\n\n-- stderr:\n{}\n", std::str::from_utf8(&output_sng_clean.stderr).expect(""));

	let res_orig = read(&orig_sng).expect("could not read orig.sng");
	let res_orig = ::std::str::from_utf8(&res_orig).expect("could not read orig.sng");
	let res_orig = &res_orig[res_orig.find('\n').expect("could not read orig.sng")..]; // Zeroth line of sng output is the original file name; exclude that line from the comparison
	let res_clean = read(&clean_sng).expect("could not read clean.sng");
	let res_clean = ::std::str::from_utf8(&res_clean).expect("could not read clean.sng");
	let res_clean = &res_clean[res_clean.find('\n').expect("could not read clean.sng")..]; // Zeroth line of sng output is the original file name; exclude that line from the comparison
	assert_eq!(res_orig, res_clean);

	::std::fs::remove_file(&orig_png).expect("could not delete temporary files");
	::std::fs::remove_file(&orig_sng).expect("could not delete temporary files");
	::std::fs::remove_file(&clean_png).expect("could not delete temporary files");
	::std::fs::remove_file(&clean_sng).expect("could not delete temporary files");
}

for_each_valid_file!(test_one, &[]);
for_each_unsafecopy_file!(test_one, &["--copy-unsafe"]);
