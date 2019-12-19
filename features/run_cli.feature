# Actually run the CLI (command-line interface) for integration testing
Feature: Run the CLI (command line interface)

  Rule: Show usage when missing configuration

    Scenario: No config file and no arguments
      Given no config file
      When I run the CLI with no args
      Then the stdout should contain:
        """
        Diff structured data files using key fields, with high performance.

        USAGE:
            experiment-differ [OPTIONS]

        FLAGS:
            -h, --help       Prints help information
            -V, --version    Prints version information

        OPTIONS:
            -c, --config <FILE>    Config file [default: example.yml]
        """
