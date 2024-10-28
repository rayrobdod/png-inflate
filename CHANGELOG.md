# Changelog

## [Unreleased]
* add --apng argument.
  When enabled, apng chunks are treated as known chunks, and fdAT chunk data is inflated.
* Adjusted some error messages to be slightly more informative
  and print in natural-language instead of the programmatic debug print style
* Include filenames in error messages
* Implement option `--assume-filename` to print a filename instead of `stdin`
  in error messages when input is stdin.
* Add PNG Third Edition W3C Candidate Recommendation Draft 18 July 2024 chunks and `sTER` to list of known chunks.
  They are all pass through, but `cICP` and `sTER` are not safe-to-copy, so images to process can include those chunks by default now.

## [v2023.6.10] 2023-06-10
Refactors, process improvements, dependency updating

No specific bug fixes or new features

## [v2021.6.12] 2021-06-12
* No longer truncate files after the first IEND chunk.
* When the destination is file, instead of directly writing to the destination file, write to a
  tempfile then move the tempfile to the destination.

## [v2019.10.10] 2019-10-10
Initial release.
