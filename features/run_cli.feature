# Actually run the CLI (command-line interface) for integration testing
Feature: Run CLI

  Scenario: No arguments and no config file
    Given no config file
    When I run the CLI with no args
    Then the output should contain:
    """
    TODO: Usage output
    """
