# Features

Our [cucumber-rust](https://github.com/bbqsrc/cucumber-rust)
feature files are organized into directories according to the
[agile testing pyramid](https://cucumber.io/blog/bdd/where_should_you_use_bdd/):
* **End-to-end:** Drives the _entire application_, using the same interface as a
  _user_.
* **Integration:** Drives a large part of the application stack usually below the UI
* Unit: Checks one unit of application in isolation often mocking other units
