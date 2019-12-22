use cucumber::cucumber;
use once_cell_regex::regex;
use std::fs::File;
use std::fs::{create_dir_all, remove_dir_all};
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use which::which;

pub struct MyWorld {
    temp_test_dir: Option<PathBuf>,
    stdout_of_run: Option<String>,
    stderr_of_run: Option<String>,
}

impl cucumber::World for MyWorld {}

impl std::default::Default for MyWorld {
    // This function is called every time a new scenario is started
    fn default() -> MyWorld {
        MyWorld {
            temp_test_dir: None,
            stdout_of_run: None,
            stderr_of_run: None,
        }
    }
}

impl Drop for MyWorld {
    fn drop(&mut self) {
        if let Some(temp_path) = self.temp_test_dir.as_ref() {
            // NOTE: Comment me out to retain temp test dir for investigation
            remove_dir_all(temp_path).unwrap();
        }
    }
}

impl MyWorld {
    fn create_temp_test_dir(&mut self, step_string: String) {
        let re_non_words = regex!(r"\W");

        let step_cleaned: String = re_non_words
            .replace_all(step_string.to_lowercase().as_str(), "-")
            .into();

        let path = PathBuf::from("./tmp/test_scenarios").join(step_cleaned);
        create_dir_all(&path).expect(format!("failed to create dir '{}'", path.display()).as_str());
        self.temp_test_dir = Some(path);
    }

    fn create_file_in_test_dir(&mut self, filename: &String, content: &String) {
        let dir_path = self
            .temp_test_dir
            .as_ref()
            .expect("no temp test dir available in step")
            .as_path();

        let file_path_buf = dir_path.join(filename);
        let file_path = file_path_buf.as_path();

        let mut file = File::create(file_path)
            .expect(format!("failed to create file '{}'", file_path.display()).as_str());

        file.write_all(content.as_bytes())
            .expect(format!("failed to write to file '{}'", file_path.display()).as_str());
    }

    fn run_and_capture(&mut self, args: Vec<&str>) {
        let cargo_path = which("cargo").expect("failed to find `cargo` in path");

        let output = Command::new(cargo_path)
            .arg("run")
            .arg("--")
            .args(args)
            .current_dir(self.temp_test_dir.as_ref().unwrap_or(&PathBuf::from(".")))
            .output()
            .expect("failed to execute process `cargo run`");

        self.stdout_of_run =
            Some(String::from_utf8(output.stdout).expect("invalid utf8 in stdout of `cargo run`"));
        self.stderr_of_run =
            Some(String::from_utf8(output.stderr).expect("invalid utf8 in stderr of `cargo run`"));
    }
}

mod example_steps {
    use cucumber::steps;
    use cucumber::Step;

    fn expect_docstring(step: &Step) -> &String {
        step.docstring()
            .expect(format!("Step missing docstring: '{:#?}'", step).as_str())
    }

    // Any type that implements cucumber::World + Default can be the world
    steps!(crate::MyWorld => {
        //
        // Run CLI steps
        //
        given "no config file" |world, step| {
            world.create_temp_test_dir(step.to_string());
        };

        given regex r"^a(n invalid)? config file named `([^`]+)` with content:$" (String, String) |world, _invalid, filename, step| {
            world.create_temp_test_dir(step.to_string());
            let content = expect_docstring(step);
            world.create_file_in_test_dir(&filename, content);
        };

        when "I run the CLI with no args" |world, _step| {
            world.run_and_capture([].to_vec());
        };

        when regex r"^I run the CLI with `([^`]+)`$" (String) |world, all_args, _step| {
            world.run_and_capture(all_args.split_whitespace().collect::<Vec<&str>>());
        };

        then "the stdout should contain:" |world, step| {
            let stdout = world.stdout_of_run.as_ref().expect("Step missing stdout of run");
            let expected = expect_docstring(step);
            assert!(stdout.contains(expected), "[stdout]\n{}\n[expected]\n{}", stdout, expected);
        };

        then "the stderr should contain:" |world, step| {
            let stderr = world.stderr_of_run.as_ref().expect("Step missing stderr of run");
            let expected = expect_docstring(step);
            assert!(stderr.contains(expected), "[stderr]\n{}\n[expected]\n{}", stderr, expected);
        };
    });
}

cucumber! {
    features: "./features", // Path to our feature files
    world: crate::MyWorld, // The world needs to be the same for steps and the main cucumber call
    steps: &[
        example_steps::steps // the `steps!` macro creates a `steps` function in a module
    ]
}
