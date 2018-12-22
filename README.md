PNG images contain deflate-compressed image data. ^(They can also contain metadata similarly compressed; the point
stands.) Small changes to the raw image data can result in in disproportionately large changes in the compressed data.
Especially if different editors with different compression schemes handle the data. In addition, compressed data does
not tend to compress well.

This program reads a PNG image from stdin and outputs that same image to stdout, but without compression. Thus, it can
act as a git clean filter. If installed as such, on a `git add` operation, the image is piped through the program, and
the resulting uncompressed image is stored in the git file store. Git does compress files places in a git file store, so
a loose file in the store should be a similar size either way. However, when the images are placed in a pack file -
either during a `gc` operation or during file transfer - the smaller delta allows git to more efficiently create a pack file.

This does not otherwise modify the image; do not mistake this for creating a canonical form. The image's bit depth,
color type, filter method and the like will not be changed, so there will still be multiple ways to represent the same image. 


It may be prudent to, if using this, to also use a smudge filter that recompresses the png image and/or add a step to a
relevant build script that compresses images. Unfortunately, pngout, pngcrush and the like all seem to have
pass-file-names-as-arguments conventions, and as such don't fit the git filter interface.

# Similar Projects

https://github.com/costerwi/rezip does the same thing, but for zip archive files
