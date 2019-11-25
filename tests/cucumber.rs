use cucumber::cucumber;
use std::fs::create_dir_all;
use std::path::PathBuf;
use std::process::Command;
use which::which;

pub struct MyWorld {
    temp_test_dir: Option<PathBuf>,
    stdout_of_run: Option<String>,
}

impl cucumber::World for MyWorld {}

impl std::default::Default for MyWorld {
    // This function is called every time a new scenario is started
    fn default() -> MyWorld {
        MyWorld {
            temp_test_dir: None,
            stdout_of_run: None,
        }
    }
}

impl MyWorld {
    fn create_temp_test_dir(&mut self) {
        let path = PathBuf::from("./tmp/test_scenarios");
        create_dir_all(&path).expect(format!("failed to create dir '{}'", path.display()).as_str());
        self.temp_test_dir = Some(path);
    }

    fn run_and_capture_stdout(&mut self, args: Vec<&str>) {
        let cargo_path = which("cargo").expect("failed to find `cargo` in path");

        let output = Command::new(cargo_path)
            .arg("run")
            .args(args)
            .current_dir(self.temp_test_dir.as_ref().unwrap_or(&PathBuf::from(".")))
            .output()
            .expect("failed to execute process `cargo run`");

        let stdout =
            String::from_utf8(output.stdout).expect("invalid utf8 in stdout of `cargo run`");

        self.stdout_of_run = Some(stdout);
    }
}

mod example_steps {
    use cucumber::steps;
    use once_cell_regex::regex;

    fn extract_quoted_args(matches: &[String]) -> Vec<&str> {
        let re = regex!(r"`([^`]+)`");

        let caps = matches
            .iter()
            .skip(1)
            .flat_map(|s| re.captures_iter(s))
            .collect::<Vec<_>>();

        caps.iter()
            .skip(1)
            .flat_map(|c| c.iter().skip(1))
            .flatten()
            .map(|m| m.as_str())
            .collect::<Vec<_>>()
    }

    // Any type that implements cucumber::World + Default can be the world
    steps!(crate::MyWorld => {
        //
        // Run CLI steps
        //
        given "no config file" |world, _step| {
            world.create_temp_test_dir();
        };

        when regex r"^I run the CLI with ((no args)|(`[^`]+`)+)$" |world, matches, _step| {
            world.run_and_capture_stdout(extract_quoted_args(matches));
        };
    });
}

// A setup function to be called before everything else
fn setup() {}

cucumber! {
    features: "./features", // Path to our feature files
    world: crate::MyWorld, // The world needs to be the same for steps and the main cucumber call
    steps: &[
        example_steps::steps // the `steps!` macro creates a `steps` function in a module
    ],
    setup: setup, // Optional; called once before everything
    before: &[],
    after: &[]
}
