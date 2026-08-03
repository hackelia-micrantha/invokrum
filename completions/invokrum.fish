complete -c invokrum -f
complete -c invokrum -n '__fish_use_subcommand' -a validate -d 'Validate a pack and optional profile'
complete -c invokrum -n '__fish_use_subcommand' -a compose -d 'Compose exact normalized context bytes'
complete -c invokrum -n '__fish_use_subcommand' -a inspect -d 'Inspect the resolved manifest'
complete -c invokrum -n '__fish_use_subcommand' -a lock -d 'Create canonical lock bytes'
complete -c invokrum -n '__fish_use_subcommand' -a verify -d 'Verify repository state against a lock'
complete -c invokrum -n '__fish_use_subcommand' -a diff -d 'Compare canonical locks'
complete -c invokrum -n '__fish_use_subcommand' -a help -d 'Show usage'
complete -c invokrum -n '__fish_use_subcommand' -a version -d 'Show version'

for command in validate compose inspect lock verify
    complete -c invokrum -n "__fish_seen_subcommand_from $command" -l pack -r -d 'Pack document'
    complete -c invokrum -n "__fish_seen_subcommand_from $command" -l profile -r -d 'Profile identifier'
end

for command in validate inspect verify diff
    complete -c invokrum -n "__fish_seen_subcommand_from $command" -l format -r -a 'human json' -d 'Output format'
end

for command in compose lock
    complete -c invokrum -n "__fish_seen_subcommand_from $command" -l output -r -d 'Output file'
    complete -c invokrum -n "__fish_seen_subcommand_from $command" -l force -d 'Replace an existing regular output file'
end

complete -c invokrum -n '__fish_seen_subcommand_from verify' -l lock -r -d 'Canonical lockfile'
complete -c invokrum -l no-color -d 'Disable color output'
complete -c invokrum -s h -l help -d 'Show usage'
complete -c invokrum -l version -d 'Show version'
