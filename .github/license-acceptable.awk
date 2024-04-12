BEGIN {FS="\t"}
{
  isAcceptable=0
  split($5, licenseOptions, " OR ")
  for (i in licenseOptions) {
    isAcceptable = isAcceptable ||
      licenseOptions[i] == "Apache-2.0" ||
      licenseOptions[i] == "BSD-3-Clause" ||
      licenseOptions[i] == "MIT" ||
      licenseOptions[i] == "license"
      # "license" is the value of the header of this TSV column
  }
  if (! isAcceptable) {
    print($1)
    print($5)
    exit 3
  }
}
