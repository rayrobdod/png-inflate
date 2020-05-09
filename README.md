# Png Inflate
[![Travis Build](https://travis-ci.org/rayrobdod/png-inflate.svg?branch=master)](https://travis-ci.org/rayrobdod/png-inflate)
[![Appveyor Build](https://ci.appveyor.com/api/projects/status/ypi8acefrievc54i/branch/master?svg=true)](https://ci.appveyor.com/project/rayrobdod/png-inflate/branch/master)

PNG images contain deflate-compressed image data. Small changes to the raw image data can result in in
disproportionately large changes in the compressed data. Especially if different editors with different compression
schemes handle the data. This makes delta compression of compressed data inefficient.

This program reads a PNG image and outputs that same image, but without compression. It can read from stdin and write to
stdout and thus can act as a git clean filter. If installed as such, on a `git add` operation, the image is piped
through the program, and the resulting uncompressed image is stored in the git file store. Git does compress files
places in a git file store, so a loose file in the store should be a similar size either way. However, when the images
are placed in a pack file - either during a `gc` operation or during file transfer - the smaller delta allows git to
more efficiently create a pack file.

This does not otherwise modify the image. The image's bit depth, color type, filter method and the like will not be
changed, so there will still be multiple ways to represent the same image.


It may be prudent to, if using this, to also use a git smudge filter that recompresses the png image and/or add a step
to a relevant build script that compresses images with a tool such as [pngout](http://www.advsys.net/ken/utils.htm) or
[pngcrush](https://pmt.sourceforge.io/pngcrush/).

# How to Install a Git Filter

These instructions assume that the binary is located at `/opt/png_inflate`, that git's `core.attributesFile` config is
not set, and that you don't mind smashing an existing git-filter named `png_inflate` (you can check for existing filters
using `git config -l | grep filter`)

To install globally

```bash
git config --global --replace-all filter.png_inflate.clean /opt/png_inflate
echo "*.png filter=png_inflate" >>${XDG_CONFIG_HOME-${HOME}/.config}/git/attributes
```

To install for a single repository

```bash
git config --local --replace-all filter.png_inflate.clean /opt/png_inflate
echo "*.png filter=png_inflate" >>.git/info/attributes
```

Go to a git directory and run `git check-attr filter -- abc.png` to check that the filter is installed properly; it
should say `png_inflate` instead of `unspecified`.

# Similar Projects

https://github.com/costerwi/rezip does the same uncompressed repack for zip archive files
