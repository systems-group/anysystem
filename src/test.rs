//! Testing facilities.

use std::collections::BTreeMap;

/// A test result.
pub type TestResult = Result<bool, String>;

pub(crate) struct Test<T> {
    name: String,
    func: fn(&T) -> TestResult,
    config: T,
}

/// A set of tests supposed to be run together.
pub struct TestSuite<T> {
    tests: Vec<Test<T>>,
}

impl<T> TestSuite<T> {
    /// Creates an empty test suite.
    pub fn new() -> Self {
        Self { tests: Vec::new() }
    }

    /// Adds a test to the suite.
    pub fn add(&mut self, name: &str, f: fn(&T) -> TestResult, config: T) {
        self.tests.push(Test {
            name: name.to_string(),
            func: f,
            config,
        });
    }

    /// Executes the test suite by running each test in turn.
    ///
    /// Collects and prints the result of each test, and prints the summary in the end.  
    /// Returns whether all tests are passed and results for each test.
    pub fn run(&mut self) -> (bool, BTreeMap<String, TestResult>) {
        let total_count = self.tests.len();
        let mut passed_count = 0;
        let mut test_results = BTreeMap::new();
        for test in &self.tests {
            println!("\n--- {} ---\n", test.name);
            let result = (test.func)(&test.config);
            test_results.insert(test.name.clone(), result.clone());
            match result {
                Ok(_) => {
                    println!("\nPASSED\n");
                    passed_count += 1;
                }
                Err(e) => {
                    println!("\nFAILED: {e}\n");
                }
            }
        }
        println!("-------------------------------------------------------------------------------");
        println!("\nPassed {passed_count} from {total_count} tests\n");
        let all_passed = passed_count == total_count;
        if !all_passed {
            println!("Failed tests:");
            for (test, result) in test_results.iter() {
                if let Err(e) = result {
                    println!("- {test}: {e}")
                }
            }
            println!();
        }
        (all_passed, test_results)
    }

    /// Runs the specified test and prints its result.
    pub fn run_test(&mut self, name: &str) {
        for test in &self.tests {
            if test.name == name {
                println!("\n--- {} ---\n", test.name);
                match (test.func)(&test.config) {
                    Ok(_) => println!("\nPASSED\n"),
                    Err(e) => println!("\nFAILED: {e}\n"),
                }
            }
        }
    }
}

impl<T> Default for TestSuite<T> {
    fn default() -> Self {
        TestSuite::new()
    }
}
