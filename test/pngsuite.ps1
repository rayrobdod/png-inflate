#!pwsh

# Set-ExecutionPolicy Unrestricted -Scope Process
$MODE_STRING=if ($env:MODE -eq '--release') {'release'} else {'debug'}

$SUITE_WEB='http://www.schaik.com/pngsuite2011/PngSuite-2017jul19.zip'
$SUITE_ZIP=$PWD.Path + '/.pngsuite.zip'
$SUITE_FILES=$PWD.Path + '/.pngsuite_files/'
$DUT=$PWD.Path + '/target/' + $MODE_STRING + '/png_inflate'
$SUITE_TARGET=$PWD.Path + '/.pngsuite_results/'

mkdir $SUITE_FILES
mkdir $SUITE_TARGET

(New-Object Net.WebClient).DownloadFile($SUITE_WEB, $SUITE_ZIP)
Expand-Archive -Path $SUITE_ZIP -DestinationPath $SUITE_FILES

$results = @();

class Test {
	$Input
	$Output
	$Errout
	$ExpectedResult
	$ActualResult
}

ForEach ($file in (dir $SUITE_FILES)) {
	if ($file.Extension -eq ".png") {
		$test = [Test]::new()
		$test.Input = $file

		$test.Output = $SUITE_TARGET + '' + $test.Input.Name + ''
		$test.Errout = $SUITE_TARGET + '' + $test.Input.Name + '.err'

		# suite files that start with an 'x' are intentionally malformed
		$test.ExpectedResult = -Not $test.Input.Name.StartsWith("x")

		&$DUT $test.Input.FullName $test.Output 2>$test.Errout
		$test.ActualResult = $?;

		# TODO: test output file for equivalence with input files?

		$results += $test
	} else {
		# Do nothing with the readme or license files
	}
}

ForEach ($result in $results) {
	$result_is_expected = if ($result.ActualResult -eq $result.ExpectedResult) {'ok'} else {'FAILED'}
	Echo $('test ' + $result.Input.Name + ' ... ' + $result_is_expected)
}

$all_result_is_expected = $true
ForEach ($result in $results) {
	$result_is_expected = $result.ActualResult -eq $result.ExpectedResult
	$all_result_is_expected = $all_result_is_expected -and $result_is_expected
	if (-Not $result_is_expected) {
		Echo ''
		#Echo $('---- Failure: ' + $result.Input.Name + ' stdout ----')
		#Get-Content $result.Output | Echo
		#Echo ''
		Echo $('---- Failure: ' + $result.Input.Name + ' stderr ----')
		Get-Content $result.ErrOut | Echo
	}
}

exit $(if ($all_result_is_expected) {0} else {1})
