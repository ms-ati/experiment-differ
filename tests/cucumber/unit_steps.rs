use cucumber::cucumber;

mod unit_steps {
    use cucumber::steps;
    use experiment_differ::*;
    use std::default::Default;
    use std::io;
    use structopt::StructOpt;
    use std::fs::File;
    use tempfile::tempfile;
    use mockall::predicate::eq;
    use std::path::PathBuf;

    #[derive(Debug, Default)]
    pub struct UnitWorld {
        given_cli_args: Option<String>
    }

    impl cucumber::World for UnitWorld {}

    impl UnitWorld {
        //
        // Setters of state from 'Given's and 'When's
        //

        fn set_given_cli_args(self: &mut UnitWorld, val: String) {
            self.given_cli_args = match val.replace("<none>", "").trim() {
                "" => None,
                s => Some(s.to_string()),
            }
        }

        fn mock_path_readability(self: &mut UnitWorld, path: &PathBuf, readability: String) {
            let guard = cli::GLOBAL_PATH_INFO_MOCKABLE.lock();
            let mut mock = Box::new(cli::MockPathInfoMockable::new());

            match readability.as_str() {
                "NotFound" => {
                    mock.expect_path_exists_mockable().with(eq(path)).returning(|| false);
                },
                "NotAFile" => {
                    mock.expect_path_exists_mockable().returning(|| true);
                    mock.expect_path_is_file_mockable().returning(|| false);
                },
                "OpenFail" => {
                    mock.expect_path_exists_mockable().returning(|| true);
                    mock.expect_path_is_file_mockable().returning(|| true);
                    mock.expect_file_open_mockable().returning(||
                        io::Result::Err(io::Error::from(io::ErrorKind::PermissionDenied))
                    );
                },
                "Readable" => {
                    mock.expect_path_exists_mockable().returning(|| true);
                    mock.expect_path_is_file_mockable().returning(|| true);
                    mock.expect_file_open_mockable().returning(|| tempfile());
                },
                _ => panic!("Unknown readability: {}", readability)
            }

            *guard = mock;
        }

        //
        // Assertions from 'Then's
        //

        fn assert_cli_args_from_iter_safe_eq(self: &UnitWorld, expected: String) {
            let args_vec: Vec<String> = self
                .given_cli_args
                .iter()
                .flat_map(|s| s.split_ascii_whitespace())
                .map(|s| s.to_string())
                .collect();
            let result = cli::Args::from_iter_safe(args_vec);
            assert_eq!(expected, format!("{:?}", result));
        }
    }

    // Any type that implements cucumber::World + Default can be the world
    steps!(UnitWorld => {
        given regex r"^command-line args: (.*)$" (String) |world, cli_args, _step| {
            world.set_given_cli_args(cli_args);
        };

        given regex r"^readability of path `([^`]+)` is: (.+)$"
            (PathBuf, String) |world, path, readability, _step| {
                world.mock_path_readability(&path, readability);
            };

        then regex r"cli::Args::from_iter_safe returns: (.*)$" (String) |world, expected, _step| {
            world.assert_cli_args_from_iter_safe_eq(expected);
        };
    });
}

cucumber! {
    features: "./features/unit", // Path to our feature files
    world: unit_steps::UnitWorld, // The world needs to be the same for steps and the main cucumber call
    steps: &[
        unit_steps::steps // the `steps!` macro creates a `steps` function in a module
    ]
}
