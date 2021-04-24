#!pwsh -File
<# For each valid test case, asserts that the dut truncates the output file #>

$MODE_STRING=if ($env:MODE -eq '--release') {'release'} else {'debug'}
$DUT=$PWD.Path + '/target/' + $MODE_STRING + '/png_inflate'
$INPUTS=$PWD.Path + '/.test_cases/valid/'

$results = @();

class Test {
	$Input
	<# One of Ok, Fail, Err #>
	$Result
	$Err
}

ForEach ($file in (dir $INPUTS)) {
	$test = [Test]::new()
	$test.Input = $file

	$dest = $(New-TemporaryFile)
	$test.Err = $(New-TemporaryFile)

	$initialSize = if ($file.Name -eq "pngsuite_PngSuite.png") {200000} else {12 * 1024}

	$destWriter = $dest.AppendText()
	$destWriter.Write('a' * $initialSize)
	$destWriter.Close()

	&$DUT $test.Input.FullName $dest.FullName 2>$test.Err
	if ($?) {
		$test.Result = if ($dest.Length -lt $initialSize) {"ok"} else {"FAILURE"}
	} else {
		$test.Result = "ERROR"
	}

	Remove-Item $dest -Force

	$results += $test
	Echo $('test ' + $test.Input.Name + ' ... ' + $test.Result)
}

Echo ''

$all_result_is_ok = $true
ForEach ($result in $results) {
	$result_is_ok = $result.Result -eq 'ok'
	$all_result_is_ok = $all_result_is_ok -and $result_is_ok
	if (-Not $result_is_ok) {
		if (-Not $(Get-Content $result.Err) -eq '') {
			Echo $('---- Failure: ' + $result.Input.Name + ' stderr ----')
			Get-Content $result.Err | Echo
			Echo ''
		}
	}
}

ForEach ($result in $results) {
	Remove-Item $result.Err -Force
}

exit $(if ($all_result_is_ok) {0} else {1})
