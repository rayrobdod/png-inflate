///!

use std::env;
use std::fs;
use std::fs::File;
use std::path::Path;
use std::path::PathBuf;
use std::vec::Vec;
use std::io::Write;

fn main() {
	let pngsuite_dir = env::current_dir().expect("").join("tests").join("PngSuite");
	let out_dir = env::var_os("OUT_DIR").unwrap();
	let mut cases_valid:Vec<PathBuf> = Vec::new();

	if let Ok(entries) = fs::read_dir(pngsuite_dir) {
		for entry in entries {
			if let Ok(entry) = entry {
				let entry = entry.path();
				if entry.extension().expect("").to_str().expect("") == "png" {
					if entry.file_stem().expect("").to_str().expect("")[0..1] != *"x" {
						cases_valid.push(entry);
					}
				}
			}
		}
	} else {
		println!("cargo:warning=Could not read pngsuite directory");
	}

	let valid_path = Path::new(&out_dir).join("cases.rs");
	let mut valid_file = File::create(valid_path).unwrap();

	valid_file.write(b"macro_rules! for_each_valid_file {
		($body:ident) => {\n").unwrap();
	for case in cases_valid {
		let casestem = case.file_stem().expect("not path").to_str().expect("not utf8");

		write!(valid_file, "\t\t\t#[allow(non_snake_case)]
			#[test]
			fn {}() {}
				$body(Path::new(\"{}\"));
			{}\n",
			casestem,
			"{",
			//case.display(),
			case.to_str().expect("not utf8").escape_default(),
			"}"
		).unwrap();
	}
	valid_file.write(b"\t\t}\n}\n").unwrap();
}
