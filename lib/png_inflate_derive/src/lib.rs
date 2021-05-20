extern crate proc_macro;

use proc_macro::{Delimiter, Spacing, Span, TokenStream, TokenTree};
use proc_macro::{Group, Ident, Literal, Punct};
use std::path::Path;
use std::path::PathBuf;
use std::vec::Vec;

macro_rules! tokens {
	( $( $x:expr ),* , ) => {{
		let mut retval = TokenStream::new();
		$({
			let x:TokenStream = $x.into();
			retval.extend(x);
		})*
		retval
	}};
}

fn case_template(case: &Path) -> TokenStream {
	let casestem = case
		.file_stem()
		.expect("no stem")
		.to_str()
		.expect("not utf8");

	let methodname = Ident::new(casestem, Span::call_site());
	let calledname = Ident::new("body", Span::call_site());
	let casevalue = Literal::string(case.to_str().expect("not utf8"));

	let methodbody: TokenStream = tokens![
		TokenTree::Punct(Punct::new('$', Spacing::Alone)),
		TokenTree::Ident(calledname),
		TokenTree::Group(Group::new(
			Delimiter::Parenthesis,
			tokens![
				"::std::path::Path::new".parse::<TokenStream>().unwrap(),
				TokenTree::Group(Group::new(
					Delimiter::Parenthesis,
					tokens![TokenTree::Literal(casevalue),]
				)),
			]
		)),
	];

	tokens![
		"#[allow(non_snake_case)]".parse::<TokenStream>().unwrap(),
		"#[test]".parse::<TokenStream>().unwrap(),
		TokenTree::Ident(Ident::new("fn", Span::call_site())),
		TokenTree::Ident(methodname),
		TokenTree::Group(Group::new(Delimiter::Parenthesis, "".parse().unwrap())),
		TokenTree::Group(Group::new(Delimiter::Brace, methodbody)),
	]
}

fn macro_template(name: &str, cases: &[PathBuf]) -> TokenStream {
	tokens![
		"#[allow(unused_macros)]".parse::<TokenStream>().unwrap(),
		"macro_rules!".parse::<TokenStream>().unwrap(),
		TokenTree::Ident(Ident::new(name, Span::call_site())),
		TokenTree::Group(Group::new(
			Delimiter::Brace,
			tokens![
				"($body:ident) =>".parse::<TokenStream>().unwrap(),
				TokenTree::Group(Group::new(
					Delimiter::Brace,
					tokens![cases
						.iter()
						.map(|case| case_template(&case))
						.collect::<TokenStream>(),]
				)),
			]
		)),
	]
}

/// Creates a set of `for_each_xxx_file` macros, each of which take the name of a method from path
/// to unit and generates a suite of `#[test]` methods which correspond to the png files in the
/// tests directories and which call the supplied path -> unit method.
#[proc_macro]
pub fn generate_for_each_files(_input: TokenStream) -> TokenStream {
	let pngsuite_dir = ::std::env::current_dir()
		.expect("")
		.join("tests")
		.join("PngSuite");

	let mut cases_valid: Vec<PathBuf> = Vec::new();
	let mut cases_badmagic: Vec<PathBuf> = Vec::new();
	let mut cases_badchecksum: Vec<PathBuf> = Vec::new();
	let mut cases_otherinvalid: Vec<PathBuf> = Vec::new(); // cases that are invalid, but not in a way that png_inflate cares about

	if let Ok(entries) = ::std::fs::read_dir(pngsuite_dir) {
		for entry in entries {
			if let Ok(entry) = entry {
				let entry = entry.path();
				if entry.extension().expect("").to_str().expect("") == "png" {
					let file_stem = entry.file_stem().expect("").to_str().expect("");
					// the file name meanings are listed in http://www.schaik.com/pngsuite2011/
					if file_stem[0..1] != *"x" {
						cases_valid.push(entry);
					} else if file_stem[0..2] == *"xs"
						|| file_stem[0..3] == *"xcr"
						|| file_stem[0..3] == *"xlf"
					{
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
		panic!("Could not read pngsuite directory");
	}

	tokens![
		macro_template("for_each_valid_file", &cases_valid),
		macro_template("for_each_badmagic_file", &cases_badmagic),
		macro_template("for_each_badchecksum_file", &cases_badchecksum),
		macro_template("for_each_otherinvalid_file", &cases_otherinvalid),
	]
}
