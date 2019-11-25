# Actually run the CLI (command-line interface) for integration testing
Feature: Run CLI

  Scenario: No arguments and no config file
    Given no config file
    When I run the CLI with no args
    Then the stderr should contain:
      """
      Error: Os { code: 2, kind: NotFound, message: "No such file or directory" }
      """
