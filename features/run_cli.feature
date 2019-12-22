Feature: Run the CLI (command line interface)

  Rule: Show usage when missing configuration

    Scenario: No config file and no arguments
      Given no config file
      When I run the CLI with no args
      Then the stdout should contain:
        """
        example-differ 0.1.0
        Diff structured data files using key fields with high performance.

        TODO: Add additional intro paragraph for the --help output here!

        USAGE:
            example-differ [OPTIONS]

        FLAGS:
            -h, --help       Prints help information
            -V, --version    Prints version information

        OPTIONS:
            -c, --config <FILE>    Config file [default: example.yml]
        """

  Rule: Print error when invalid configuration

    Scenario: Empty default config file
      Given an invalid config file named `example.yml` with content:
        """
        """
      When I run the CLI with no args
      Then the stderr should contain:
        """
        Error: "Failed to parse example.yml: EOF while parsing a value"
        """
