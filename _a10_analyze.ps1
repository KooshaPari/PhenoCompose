$branches = @(
    'chore/l2-23-taskfile-justfile-2026-06-11',
    'chore/l2-28-hygiene-baselines-2026-06-11',
    'chore/l2-29-dependabot-2026-06-11',
    'chore/l2-32-ci-hardening-2026-06-11',
    'chore/l2-34-secret-scan-2026-06-11',
    'chore/l2-35-scorecard-renovate-2026-06-11',
    'chore/l2-36-license-changelog-2026-06-11',
    'chore/l3-43-phenocompose-cov-2026-06-11-impl',
    'chore/l4-63-phenocompose-hex-2026-06-11',
    'chore/l4-71-phenocompose-merge-2026-06-11',
    'chore/l4-71-phenocompose-pine-merge-2026-06-11',
    'chore/l5-83-phenocompose-integration-2026-06-11',
    'chore/l5-87-spec-arch-2026-06-11',
    'chore/l5-88-focus-repo-readme-agents-2026-06-11'
)

$mergedRemote = git branch -r --merged main

foreach ($b in $branches) {
    $ref = 'origin/' + $b
    $isMerged = $false
    foreach ($mr in $mergedRemote) {
        $trimmed = $mr.Trim()
        if ($trimmed -eq $ref) {
            $isMerged = $true
            break
        }
    }
    $log = git log -1 --format='%ci %h' $ref 2>&1
    $logStr = $log -join ' '
    Write-Output ("$b | merged=$isMerged | $logStr")
}
