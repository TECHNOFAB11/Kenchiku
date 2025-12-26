function _kenchiku_cmd_0
    set 1 $argv[1]
    kenchiku completion-data --scaffolds
end

function _kenchiku_cmd_1
    set 1 $argv[1]
    kenchiku completion-data --patches
end

function _kenchiku_cmd_2
    set 1 $argv[1]
    __fish_complete_path "$1"
end

function __complgen_match
    set prefix $argv[1]

    set candidates
    set descriptions
    while read c
        set a (string split --max 1 -- "	" $c)
        set --append candidates $a[1]
        if set --query a[2]
            set --append descriptions $a[2]
        else
            set --append descriptions ""
        end
    end

    if test -z "$candidates"
        return 1
    end

    set escaped_prefix (string escape --style=regex -- $prefix)
    set regex "^$escaped_prefix.*"

    set matches_case_sensitive
    set descriptions_case_sensitive
    for i in (seq 1 (count $candidates))
        if string match --regex --quiet --entire -- $regex $candidates[$i]
            set --append matches_case_sensitive $candidates[$i]
            set --append descriptions_case_sensitive $descriptions[$i]
        end
    end

    if set --query matches_case_sensitive[1]
        for i in (seq 1 (count $matches_case_sensitive))
            printf '%s	%s\n' $matches_case_sensitive[$i] $descriptions_case_sensitive[$i]
        end
        return 0
    end

    set matches_case_insensitive
    set descriptions_case_insensitive
    for i in (seq 1 (count $candidates))
        if string match --regex --quiet --ignore-case --entire -- $regex $candidates[$i]
            set --append matches_case_insensitive $candidates[$i]
            set --append descriptions_case_insensitive $descriptions[$i]
        end
    end

    if set --query matches_case_insensitive[1]
        for i in (seq 1 (count $matches_case_insensitive))
            printf '%s	%s\n' $matches_case_insensitive[$i] $descriptions_case_insensitive[$i]
        end
        return 0
    end

    return 1
end


function _kenchiku
    set COMP_LINE (commandline --cut-at-cursor)

    set COMP_WORDS
    echo $COMP_LINE | read --tokenize --array COMP_WORDS
    if string match --quiet --regex '.*\s$' $COMP_LINE
        set COMP_CWORD (math (count $COMP_WORDS) + 1)
    else
        set COMP_CWORD (count $COMP_WORDS)
    end

    set literals -v show --json list --json construct --output -y --set --force patch mcp --help

    set descrs
    set descrs[1] "Increases verbosity/decreases log level. -v -> info, -vv -> debug, -vvv -> trace"
    set descrs[2] "Shows details about a scaffold"
    set descrs[3] "Output in JSON format"
    set descrs[4] "Lists all found scaffolds"
    set descrs[5] "Runs the specified scaffolds' construction"
    set descrs[6] "Path to construct or patch in"
    set descrs[7] "Increases auto-accept level for potentially dangerous actions"
    set descrs[8] "Sets values before running (= separated, eg. 'a=b')"
    set descrs[9] "Overwrite existing files in output dir"
    set descrs[10] "Runs the specified patch"
    set descrs[11] "Starts a MCP server"
    set descrs[12] "Show help"
    set descr_literal_ids 1 2 3 4 5 6 7 8 9 10 11 12 13
    set descr_ids 1 2 3 4 3 5 6 7 8 9 10 11 12
    set regexes 
    set literal_transitions_inputs
    set nontail_transitions
    set literal_transitions_inputs[1] "1 2 4 6 11 12 13"
    set literal_transitions_tos[1] "2 3 4 5 6 7 7"
    set literal_transitions_inputs[2] "1 2 4 6 11 12"
    set literal_transitions_tos[2] "2 3 4 5 6 7"
    set literal_transitions_inputs[4] 5
    set literal_transitions_tos[4] 7
    set literal_transitions_inputs[8] "7 8 9 10"
    set literal_transitions_tos[8] "10 8 11 8"
    set literal_transitions_inputs[9] "7 8 9"
    set literal_transitions_tos[9] "12 9 13"
    set literal_transitions_inputs[14] 5
    set literal_transitions_tos[14] 7

    set match_anything_transitions_from 5 6 10 11 13 12 3
    set match_anything_transitions_to 8 9 8 8 9 9 14

    set state 1
    set word_index 2
    while test $word_index -lt $COMP_CWORD
        set -- word $COMP_WORDS[$word_index]

        if set --query literal_transitions_inputs[$state] && test -n $literal_transitions_inputs[$state]
            set inputs (string split ' ' $literal_transitions_inputs[$state])
            set tos (string split ' ' $literal_transitions_tos[$state])

            set literal_id (contains --index -- "$word" $literals)
            if test -n "$literal_id"
                set index (contains --index -- "$literal_id" $inputs)
                set state $tos[$index]
                set word_index (math $word_index + 1)
                continue
            end
        end

        if set --query match_anything_transitions_from[$state] && test -n $match_anything_transitions_from[$state]
            set index (contains --index -- "$state" $match_anything_transitions_from)
            set state $match_anything_transitions_to[$index]
            set word_index (math $word_index + 1)
            continue
        end

        return 1
    end

    set literal_froms_level_0 2 4 14 1 8 9
    set literal_inputs_level_0 "1 2 4 6 11 12|5|5|1 2 4 6 11 12 13|7 8 9 10|7 8 9"
    set nontail_command_froms_level_0 
    set nontail_commands_level_0 
    set nontail_regexes_level_0 
    set command_froms_level_0 6 5 10 3 12
    set commands_level_0 "1" "0" "2" "0" "2"

    for fallback_level in (seq 0 0)
        set candidates
        set froms_name literal_froms_level_$fallback_level
        set froms $$froms_name
        set index (contains --index -- "$state" $froms)
        if test -n "$index"
            set level_inputs_name literal_inputs_level_$fallback_level
            set input_assoc_values (string split '|' $$level_inputs_name)
            set state_inputs (string split ' ' $input_assoc_values[$index])
            for literal_id in $state_inputs
                set descr_index (contains --index -- "$literal_id" $descr_literal_ids)
                if test -n "$descr_index"
                    set --append candidates (printf '%s\t%s\n' $literals[$literal_id] $descrs[$descr_ids[$descr_index]])
                else
                    set --append candidates (printf '%s\n' $literals[$literal_id])
                end
            end
        end

        set commands_name command_froms_level_$fallback_level
        set commands $$commands_name
        set index (contains --index -- "$state" $commands)
        if test -n "$index"
            set commands_name commands_level_$fallback_level
            set commands (string split ' ' $$commands_name)
            set function_id $commands[$index]
            set function_name _kenchiku_cmd_$function_id
            set --append candidates ($function_name "$COMP_WORDS[$COMP_CWORD]")
        end
        printf '%s\n' $candidates | __complgen_match $COMP_WORDS[$word_index] && return 0
    end
end

complete --erase kenchiku
complete --command kenchiku --no-files --arguments "(_kenchiku)"
