# Remove the tre.exe aliases module created by the previous invocation
# if it exists
if (Get-Module tre_aliases_$env:USERNAME)
{
    Remove-Module -Force tre_aliases_$env:USERNAME
}

# Call tre.exe with the args passed to this script and the editor flag
tre.exe $args -e

# Import the new aliases module created by tre.exe
Import-Module $Env:TEMP\tre_aliases_$env:USERNAME.psm1
