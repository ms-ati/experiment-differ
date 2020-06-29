Feature: cli::Args

  Rule: Arg `--file` is a readable file

    Scenario Outline: Arg `--file` examples
      Given command-line args: <args>
      And readability of path `<path>` is: <status>
      Then cli::Args::from_iter_safe returns: <expected>

      Examples:
        | args   | path      | readability | expected                        |
        | <none> | trecs.yml | missing     | Err("File trecs.yml not found") |
        | <none> | trecs.yml | readable    | Err("File trecs.yml not found") |
