//! For each valid test case, asserts that the dut creates semantically-identical files

extern crate png;
extern crate png_inflate_derive;
extern crate tempfile;

use png::Info;
use png_inflate_derive::generate_for_each_files;
use std::fs::File;
use std::path::Path;
use std::process::Command;
use tempfile::NamedTempFile;

const PROGRAM_EXE: &str = env!("CARGO_BIN_EXE_png_inflate");

generate_for_each_files!();

fn assert_option_equals<A, F>(left: &Option<A>, right: &Option<A>, f: F)
where
	F: FnOnce(&A, &A),
{
	match (left, right) {
		(Some(left), Some(right)) => {
			f(left, right);
		},
		(None, None) => {
			// do nothing
		},
		_ => {
			panic!("left and right were one defined and one not defined");
		},
	}
}

fn assert_pixel_dims_equals(left: &png::PixelDimensions, right: &png::PixelDimensions) {
	assert!(left.xppu == right.xppu);
	assert!(left.yppu == right.yppu);
	assert!(left.unit == right.unit);
}

fn assert_anim_control_equals(left: &png::AnimationControl, right: &png::AnimationControl) {
	assert!(left.num_frames == right.num_frames);
	assert!(left.num_plays == right.num_plays);
}

fn assert_frame_control_equals(left: &png::FrameControl, right: &png::FrameControl) {
	assert!(left.sequence_number == right.sequence_number);
	assert!(left.width == right.width);
	assert!(left.height == right.height);
	assert!(left.x_offset == right.x_offset);
	assert!(left.y_offset == right.y_offset);
	assert!(left.delay_num == right.delay_num);
	assert!(left.delay_den == right.delay_den);
	assert!(left.dispose_op == right.dispose_op);
	assert!(left.blend_op == right.blend_op);
}

fn assert_info_equals(left: &Info, right: &Info) {
	assert!(left.width == right.width);
	assert!(left.height == right.height);
	assert!(left.bit_depth == right.bit_depth);
	assert!(left.color_type == right.color_type);
	assert!(left.interlaced == right.interlaced);
	assert!(left.trns == right.trns);
	assert_option_equals(
		&left.pixel_dims,
		&right.pixel_dims,
		assert_pixel_dims_equals,
	);
	assert!(left.palette == right.palette);
	assert!(left.gama_chunk == right.gama_chunk);
	assert!(left.chrm_chunk == right.chrm_chunk);
	assert_option_equals(
		&left.frame_control,
		&right.frame_control,
		assert_frame_control_equals,
	);
	assert_option_equals(
		&left.animation_control,
		&right.animation_control,
		assert_anim_control_equals,
	);
	assert!(left.source_gamma == right.source_gamma);
	assert!(left.source_chromaticities == right.source_chromaticities);
	assert!(left.srgb == right.srgb);
	assert!(left.icc_profile == right.icc_profile);
	assert!(left.uncompressed_latin1_text == right.uncompressed_latin1_text);

	// ZTXtChunk has an equals, but it seems to depend on the text's compressed representation
	assert!(left.compressed_latin1_text.len() == right.compressed_latin1_text.len());
	for (left_chunk, right_chunk) in left
		.compressed_latin1_text
		.iter()
		.zip(right.compressed_latin1_text.iter())
	{
		assert!(left_chunk.keyword == right_chunk.keyword);
		assert!(left_chunk.get_text().unwrap() == right_chunk.get_text().unwrap());
	}

	assert!(left.utf8_text.len() == right.utf8_text.len());
	for (left_chunk, right_chunk) in left.utf8_text.iter().zip(right.utf8_text.iter()) {
		assert!(left_chunk.keyword == right_chunk.keyword);
		assert!(left_chunk.language_tag == right_chunk.language_tag);
		assert!(left_chunk.translated_keyword == right_chunk.translated_keyword);
		assert!(left_chunk.get_text().unwrap() == right_chunk.get_text().unwrap());
	}
}

fn test_one(infile: &Path, extra_args: &[&str]) {
	let cleanfile = NamedTempFile::new().expect("");
	let cleanfile = cleanfile.into_temp_path();

	let output_inflate = Command::new(PROGRAM_EXE)
		.arg(infile)
		.arg(&cleanfile)
		.args(extra_args)
		.output()
		.expect("failed to execute png_inflate process");
	assert!(
		output_inflate.status.success(),
		"png_inflate execution was not success\n\n-- stderr:\n{}\n",
		std::str::from_utf8(&output_inflate.stderr).expect("")
	);

	let mut input_decoder = png::Decoder::new(File::open(infile).unwrap());
	let mut clean_decoder = png::Decoder::new(File::open(cleanfile).unwrap());
	assert_info_equals(
		input_decoder.read_header_info().unwrap(),
		clean_decoder.read_header_info().unwrap(),
	);

	let mut input_reader = input_decoder.read_info().unwrap();
	let mut clean_reader = clean_decoder.read_info().unwrap();
	assert_info_equals(input_reader.info(), clean_reader.info());

	let mut input_buffer = vec![0; input_reader.output_buffer_size()];
	let mut clean_buffer = vec![0; clean_reader.output_buffer_size()];
	let frame_count = input_reader
		.info()
		.animation_control()
		.map(|x| x.num_frames)
		.unwrap_or(1);
	for _ in 0..frame_count {
		let input_frame_info = input_reader.next_frame(&mut input_buffer);
		let clean_frame_info = clean_reader.next_frame(&mut clean_buffer);
		assert!(input_frame_info.unwrap() == clean_frame_info.unwrap());
		assert!(input_buffer == clean_buffer);
		assert_info_equals(input_reader.info(), clean_reader.info());
	}
}

mod noargs {
	for_each_valid_file!(super::test_one, &[]);
}
mod copy_unsafe {
	for_each_unsafecopy_file!(super::test_one, &["--copy-unsafe"]);
}
mod apng {
	for_each_apng_file!(super::test_one, &["--apng"]);
}
