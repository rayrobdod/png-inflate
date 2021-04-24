///!

use std::env;
use std::fs;
use std::fs::File;
use std::path::Path;
use std::path::PathBuf;
use std::vec::Vec;
use std::io::Write;

fn write_fn_template(out_file:&mut File, case:&Path) {
	let casestem = case.file_stem().expect("no stem").to_str().expect("not utf8");

	write!(out_file, "\t\t#[allow(non_snake_case)]
		#[test]
		fn {}() {}
			$body(Path::new(\"{}\"));
		{}\n",
		casestem,
		"{",
		case.to_str().expect("not utf8").escape_default(),
		"}"
	).unwrap();
}

fn main() {
	let pngsuite_dir = env::current_dir().expect("").join("tests").join("PngSuite");
	let out_dir = env::var_os("OUT_DIR").unwrap();
	let mut cases_valid:Vec<PathBuf> = Vec::new();
	let mut cases_badmagic:Vec<PathBuf> = Vec::new();
	let mut cases_badchecksum:Vec<PathBuf> = Vec::new();
	let mut cases_otherinvalid:Vec<PathBuf> = Vec::new(); // cases that are invalid, but not in a way that png_inflate cares about

	if let Ok(entries) = fs::read_dir(pngsuite_dir) {
		for entry in entries {
			if let Ok(entry) = entry {
				let entry = entry.path();
				if entry.extension().expect("").to_str().expect("") == "png" {
					let file_stem = entry.file_stem().expect("").to_str().expect("");
					// the file name meanings are listed in http://www.schaik.com/pngsuite2011/
					if file_stem[0..1] != *"x" {
						cases_valid.push(entry);
					} else if file_stem[0..2] == *"xs" || file_stem[0..3] == *"xcr" || file_stem[0..3] == *"xlf" {
						cases_badmagic.push(entry);
					} else if file_stem[0..3] == *"xhd" || file_stem[0..3] == *"xcs" {
						cases_badchecksum.push(entry);
					} else {
						cases_otherinvalid.push(entry);
					}
				}
			}
		}
	} else {
		println!("cargo:warning=Could not read pngsuite directory");
	}

	let valid_path = Path::new(&out_dir).join("cases.rs");
	let mut valid_file = File::create(valid_path).unwrap();

	valid_file.write(b"#[allow(unused_macros)]
		macro_rules! for_each_valid_file {
			($body:ident) => {
			").unwrap();
	for case in cases_valid {
		write_fn_template(&mut valid_file, &case);
	}
	valid_file.write(b"\t}\n}\n\n").unwrap();

	valid_file.write(b"#[allow(unused_macros)]
		macro_rules! for_each_badmagic_file {
		($body:ident) => {
		").unwrap();
	for case in cases_badmagic {
		write_fn_template(&mut valid_file, &case);
	}
	valid_file.write(b"\t}\n}\n").unwrap();

	valid_file.write(b"#[allow(unused_macros)]
		macro_rules! for_each_badchecksum_file {
		($body:ident) => {
		").unwrap();
	for case in cases_badchecksum {
		write_fn_template(&mut valid_file, &case);
	}
	valid_file.write(b"\t}\n}\n").unwrap();

	valid_file.write(b"#[allow(unused_macros)]
		macro_rules! for_each_otherinvalid_file {
		($body:ident) => {
		").unwrap();
	for case in cases_otherinvalid {
		write_fn_template(&mut valid_file, &case);
	}
	valid_file.write(b"\t}\n}\n").unwrap();
}
