use cucumber::cucumber;
use std::default::Default;

pub struct MyWorld {
    _foo: u32
}

impl cucumber::World for MyWorld {}

impl Default for MyWorld {
    // This function is called every time a new scenario is started
    fn default() -> MyWorld {
        MyWorld {
            _foo: 0
        }
    }
}

mod example_steps {
    use cucumber::steps;
    use cucumber::Step;

    fn _expect_docstring(step: &Step) -> &String {
        step.docstring()
            .expect(format!("Step missing docstring: '{:#?}'", step).as_str())
    }

    // Any type that implements cucumber::World + Default can be the world
    steps!(crate::MyWorld => {
        given "foo" |_world, _step| {
        };

        then "bar" |_world, _step| {
            assert!(true);
        };
    });
}

cucumber! {
    features: "./features/unit", // Path to our feature files
    world: crate::MyWorld, // The world needs to be the same for steps and the main cucumber call
    steps: &[
        example_steps::steps // the `steps!` macro creates a `steps` function in a module
    ]
}
