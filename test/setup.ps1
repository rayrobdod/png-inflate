#!pwsh -File
<# Downloads sample PNG files and sorts them by validity #>

$PNGSUITE_WEB='http://www.schaik.com/pngsuite2011/PngSuite-2017jul19.zip'
$PNGSUITE_LOCAL=$(New-TemporaryFile).FullName + '.zip'
$PNGSUITE_EXPLODE=$PWD.Path + '/.pngsuite/'

$TARGET=$PWD.Path + '/.test_cases/'
$TARGET_VALID=$TARGET + '/valid/'
$TARGET_INVALID=$TARGET + '/invalid/'

if (-Not $(Test-Path $TARGET)) { mkdir $TARGET }
if (-Not $(Test-Path $TARGET_VALID)) { mkdir $TARGET_VALID }
if (-Not $(Test-Path $TARGET_INVALID)) { mkdir $TARGET_INVALID }

if (-Not $(Test-Path $PNGSUITE_EXPLODE)) {
	mkdir $PNGSUITE_EXPLODE
	(New-Object Net.WebClient).DownloadFile($PNGSUITE_WEB, $PNGSUITE_LOCAL)
	Expand-Archive -Path $PNGSUITE_LOCAL -DestinationPath $PNGSUITE_EXPLODE
	Remove-Item $PNGSUITE_LOCAL -Force
}

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
