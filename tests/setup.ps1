#!pwsh -File
<# Downloads sample PNG files and sorts them by validity #>

$PNGSUITE_EXPLODE=$PWD.Path + '/tests/PngSuite/'

$TARGET=$PWD.Path + '/.test_cases/'
$TARGET_VALID=$TARGET + '/valid/'
$TARGET_INVALID=$TARGET + '/invalid/'

if (-Not $(Test-Path $TARGET)) { mkdir $TARGET | Out-Null }
if (-Not $(Test-Path $TARGET_VALID)) { mkdir $TARGET_VALID | Out-Null }
if (-Not $(Test-Path $TARGET_INVALID)) { mkdir $TARGET_INVALID | Out-Null }

ForEach ($file in (dir $PNGSUITE_EXPLODE)) {
	if ($file.Extension -eq ".png") {
		if ($file.Name.StartsWith('x')) {
			Copy-Item $file.FullName ($TARGET_INVALID + 'pngsuite_' + $file.Name)
		} else {
			Copy-Item $file.FullName ($TARGET_VALID + 'pngsuite_' + $file.Name)
		}
	} else {
		# Do nothing with the readme or license files
	}
}

# TODO: more cases?
