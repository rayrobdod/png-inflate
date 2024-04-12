# Changelog

## [Unreleased]
* Adjusted some error messages to be slightly more informative
  and print in natural-language instead of the programatic debug print style

## [v2023.6.10] 2023-06-10
Refactors, process improvements, dependency updating

No specific bug fixes or new features

## [v2021.6.12] 2021-06-12
* No longer truncate files after the first IEND chunk.
* When the destination is file, instead of directly writing to the destination file, write to a
  tempfile then move the tempfile to the destination.

## [v2019.10.10] 2019-10-10
Initial release.
