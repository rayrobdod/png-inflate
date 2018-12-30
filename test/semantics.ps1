#!pwsh -File
<# For each valid test case, asserts that the dut creates semantically-identical files #>

$MODE_STRING=if ($env:MODE -eq '--release') {'release'} else {'debug'}
$DUT=$PWD.Path + '/target/' + $MODE_STRING + '/png_inflate'
$INPUTS=$PWD.Path + '/.test_cases/valid/'
$SNG=if ($SNG -eq $null) {'sng'} else {$SNG}

$results = @();

class Test {
	$Input
	<# One of Ok, Fail, Err #>
	$Result
	$Delta
	$ErrClean
}

ForEach ($file in (dir $INPUTS)) {
	$test = [Test]::new()
	$test.Input = $file

	$clean = $(New-TemporaryFile).FullName
	$clean_png = $clean + ".png"
	$clean_sng = $clean + ".sng"
	$orig = $(New-TemporaryFile).FullName
	$orig_png = $orig + ".png"
	$orig_sng = $orig + ".sng"
	$test.Delta = $(New-TemporaryFile)
	$test.ErrClean = $(New-TemporaryFile)

	Copy-Item $file.FullName $orig_png

	&$DUT $orig_png $clean_png 2>$test.ErrClean
	if ($?) {
		# SNG acts on each command line parameter in turn, but whatever version of sng the travis build uses has problems if you do
		&$SNG $clean_png
		&$SNG $orig_png

		# Zeroth line of sng output is the original file name; exclude that line from the comparison
		$clean_contents = $(Get-Content $clean_sng)
		$clean_contents = $clean_contents[1..$clean_contents.Length]
		$orig_contents = $(Get-Content $orig_sng)
		$orig_contents = $orig_contents[1..$orig_contents.Length]
		$compare = Compare-Object $clean_contents $orig_contents

		Echo $compare >$test.Delta
		$test.Result = if ($compare) {"FAILURE"} else {"ok"}
	} else {
		$test.Result = "ERROR"
	}

	Remove-Item $clean -Force
	Remove-Item $clean_png -Force
	Remove-Item $clean_sng -Force
	Remove-Item $orig -Force
	Remove-Item $orig_png -Force
	Remove-Item $orig_sng -Force

	$results += $test
	Echo $('test ' + $test.Input.Name + ' ... ' + $test.Result)
}

Echo ''

$all_result_is_ok = $true
ForEach ($result in $results) {
	$result_is_ok = $result.Result -eq 'ok'
	$all_result_is_ok = $all_result_is_ok -and $result_is_ok
	if (-Not $result_is_ok) {
		if (-Not $(Get-Content $result.Delta) -eq '') {
			Echo $('---- Failure: ' + $result.Input.Name + ' delta ----')
			Get-Content $result.Delta | Echo
			Echo ''
		}
		if (-Not $(Get-Content $result.ErrClean) -eq '') {
			Echo $('---- Failure: ' + $result.Input.Name + ' stderr ----')
			Get-Content $result.ErrClean | Echo
			Echo ''
		}
	}
}

ForEach ($result in $results) {
	Remove-Item $result.Delta -Force
	Remove-Item $result.ErrClean -Force
}

exit $(if ($all_result_is_ok) {0} else {1})
