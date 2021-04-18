# Changelog

## [Unreleased]
* No longer truncate files after the first IEND chunk.
* When the destination is file, instead of directly writing to the destination file, write to a
  tempfile then move the tempfile to the destination.

## [v20191010] 2019-10-10
Initial release.
