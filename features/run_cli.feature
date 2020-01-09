Feature: Run the CLI (command line interface)

  Rule: Transcode a single input file

    Scenario: Transcode a JSONL file to CSV on STDOUT
      Given a config file named `example.yml` containing:
        """
        transcode:
          input:
            path: input.jsonl
          output:
            format: csv
        """
      And a file named `input.jsonl` containing:
        """
        {"a":1,"b":"one"}
        {"a":2,"b":"two"}
        """
      When I run the CLI with no args
#      Then the stdout should contain:
#         """
#         a,b
#         1,one
#         2,two
#         """

  Rule: Show usage when missing configuration

    Scenario: No config file and no arguments
      Given no config file
      When I run the CLI with no args
      Then the stdout should contain:
        """
        experiment-differ 0.1.0
        Diff structured data files using key fields with high performance.

        TODO: Add additional intro paragraph for the --help output here!

        USAGE:
            experiment-differ [OPTIONS]

        FLAGS:
            -h, --help       Prints help information
            -V, --version    Prints version information

        OPTIONS:
            -c, --config <FILE>    Config file [default: example.yml]
        """

  Rule: Print error when invalid configuration

    Scenario: Empty default config file
      Given an empty config file named `example.yml`
      When I run the CLI with no args
      Then the stderr should contain:
        """
        Error: "Failed to parse example.yml: EOF while parsing a value"
        """

    Scenario: Invalid non-default config file
      Given an invalid config file named `non-default.yml` containing:
        """
        Invalid YAML
        """
      When I run the CLI with `-c non-default.yml`
      Then the stderr should contain:
        """
        Error: "Failed to parse non-default.yml: invalid type: string \"Invalid YAML\", expected struct DifferConfig at line 1 column 1"
        """
