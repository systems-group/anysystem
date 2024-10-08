use rstest::rstest;
use std::collections::HashSet;
use std::env;
use sugars::boxed;

use anysystem::process::StringProcessState;
use anysystem::python::PyProcessFactory;
use anysystem::{Message, System};

use anysystem::mc::predicates::{goals, invariants, prunes};
use anysystem::mc::strategies::{Bfs, Dfs};
use anysystem::mc::{ModelChecker, StrategyConfig};
use anysystem::run_mc;

fn build_system(impl_file: &str) -> System {
    let mut sys = System::new(12345);
    sys.add_node("server-node");
    sys.add_node("client-node");

    let py_path = [env::current_dir().unwrap().to_str().unwrap(), "/python"].join("");
    env::set_var("PYTHONPATH", py_path);
    let impl_path = env::var("PYTHONPATH").unwrap() + "/../tests/python/" + impl_file;

    let server_f = PyProcessFactory::new(impl_path.as_str(), "PingServer");
    let server = boxed!(server_f.build(("server",), 12345));
    let client_f = PyProcessFactory::new(impl_path.as_str(), "PingClient");
    let client = boxed!(client_f.build(("client", "server"), 12345));

    sys.add_process("server", server, "server-node");
    sys.add_process("client", client, "client-node");
    sys
}

#[rstest]
#[case("dfs")]
#[case("bfs")]
fn python_normal(#[case] strategy_name: &str) {
    let system = build_system("retry.py");
    let data = r#"{"value": 0}"#.to_string();
    let messages_expected = HashSet::<String>::from_iter([data.clone()]);

    let strategy_config = StrategyConfig::default()
        .prune(prunes::sent_messages_limit(4))
        .goal(goals::got_n_local_messages("client-node", "client", 1))
        .invariant(invariants::all_invariants(vec![
            invariants::received_messages("client-node", "client", messages_expected),
            invariants::state_depth(20),
        ]));

    let result = run_mc!(&system, strategy_config, strategy_name, |system| {
        system.send_local_message("client-node", "client", Message::new("PING", &data));
    });
    assert!(result.is_ok());

    let state = system
        .get_node("client-node")
        .unwrap()
        .get_process("client")
        .unwrap()
        .state()
        .unwrap();
    let state = state.downcast_ref::<StringProcessState>().unwrap();
    assert_eq!(state, "");
}

#[rstest]
#[case("dfs")]
#[case("bfs")]
#[should_panic(expected = "Error when calling process")]
fn python_runtime_error(#[case] strategy_name: &str) {
    let system = build_system("retry_runtime_error.py");
    let data = r#"{"value": 0}"#.to_string();
    let data2 = r#"{"value": 1}"#.to_string();
    let messages_expected = HashSet::<String>::from_iter([data.clone(), data2.clone()]);

    let strategy_config = StrategyConfig::default()
        .prune(prunes::sent_messages_limit(4))
        .goal(goals::got_n_local_messages("client-node", "client", 2))
        .invariant(invariants::all_invariants(vec![
            invariants::received_messages("client-node", "client", messages_expected),
            invariants::state_depth(20),
        ]));

    let _res = run_mc!(&system, strategy_config, strategy_name, |system| {
        system.send_local_message("client-node", "client", Message::new("PING", &data));
        system.send_local_message("client-node", "client", Message::new("PING", &data2));
    });
}
