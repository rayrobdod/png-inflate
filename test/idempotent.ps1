#!pwsh -File
<# For each valid test case, asserts that the dut is idempotent #>

$MODE_STRING=if ($env:MODE -eq '--release') {'release'} else {'debug'}
$DUT=$PWD.Path + '/target/' + $MODE_STRING + '/png_inflate'
$INPUTS=$PWD.Path + '/.test_cases/valid/'

$results = @();

class Test {
	$Input
	<# One of Ok, Fail, Err #>
	$Result
	$Err1
	$Err2
}

ForEach ($file in (dir $INPUTS)) {
	$test = [Test]::new()
	$test.Input = $file

	$out1 = $(New-TemporaryFile)
	$out2 = $(New-TemporaryFile)
	$test.Err1 = $(New-TemporaryFile)
	$test.Err2 = $(New-TemporaryFile)

	&$DUT $test.Input.FullName $out1.FullName 2>$test.Err1
	if ($?) {
		&$DUT $out1.FullName $out2.FullName 2>$test.Err2
		if ($?) {
			$test.Result = if (Compare-Object -Property Hash $(Get-FileHash $out1) $(Get-FileHash $out2)) {"FAILURE"} else {"ok"}
		} else {
			$test.Result = "ERROR"
		}
	} else {
		$test.Result = "ERROR"
	}

	Remove-Item $out1 -Force
	Remove-Item $out2 -Force

	$results += $test
	Echo $('test ' + $test.Input.Name + ' ... ' + $test.Result)
}

Echo ''

$all_result_is_ok = $true
ForEach ($result in $results) {
	$result_is_ok = $result.Result -eq 'ok'
	$all_result_is_ok = $all_result_is_ok -and $result_is_ok
	if (-Not $result_is_ok) {
		if (-Not $(Get-Content $result.Err1) -eq '') {
			Echo $('---- Failure: ' + $result.Input.Name + ' stderr 1 ----')
			Get-Content $result.Err1 | Echo
			Echo ''
		}
		if (-Not $(Get-Content $result.Err2) -eq '') {
			Echo $('---- Failure: ' + $result.Input.Name + ' stderr 2 ----')
			Get-Content $result.Err2 | Echo
			Echo ''
		}
	}
}

ForEach ($result in $results) {
	Remove-Item $result.Err1 -Force
	Remove-Item $result.Err2 -Force
}

exit $(if ($all_result_is_ok) {0} else {1})
