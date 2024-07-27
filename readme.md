# AnySystem


[![Crates.io](https://img.shields.io/crates/v/anysystem)](https://crates.io/crates/anysystem)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue)](LICENSE-APACHE)
[![License](https://img.shields.io/badge/license-MIT-blue?)](LICENSE-MIT)
[![Build Status](https://github.com/systems-group/anysystem/actions/workflows/CI.yml/badge.svg?branch=main)](https://github.com/systems-group/anysystem/actions?query=workflow%3ACI+branch%3Amain)
[![Documentation](https://img.shields.io/docsrs/anysystem)](https://docs.rs/anysystem)

AnySystem is a framework for deterministic simulation and testing of distributed systems. It allows to model arbitrary systems represented as a set of _processes_ running on a set of _nodes_ connected by a _network_.

The processes communicate with each other by sending and receiving _messages_. A message between processes located on different nodes is transmitted over the network. A process can also receive and send _local messages_ which can be used to model the interaction with external entities such as users. Finally, a process can schedule _timers_ to call itself after a specified delay. You can implement arbitrary logic for processing of incoming messages and timers for each process. In addition to Rust, it is possible to implement processes in Python.

The system is built by creating the required nodes and processes, and binding each process to some node. Then it is possible to send some messages to initiate the system execution. The execution is implemented as a step-by-step _simulation_. Each step corresponds to the occurrence of some event such as message delivery or timer firing. The events are processed in the order of their timestamps by advancing the simulation time and calling a corresponding process. In response, the process can perform actions that produce new events, e.g. sending a message can produce a new message delivery event. Currently, the process execution time is not modeled, i.e. it is assumed that events are processed instantaneously. The system execution is deterministic by using a common RNG seeded with a user-defined value.

AnySystem supports modeling of typical situations found in distributed systems, such as message delays, network failures and node crashes. The provided network model can be configured to introduce fixed or random transmission delays, message loss, duplication or corruption. It is also possible to model node disconnections, control the network between each pair of nodes and in each direction, or introduce network partitions. A node can be crashed by disconnecting it from the network and stopping all processes running on it. A crashed node can be recovered later by restarting its processes and connecting it to the network.

The described features can be used for testing any distributed algorithm or application implemented as a set of processes. Step-by-step simulation allows to precisely control the system execution and produce different execution scenarios by sending messages and introducing failures at specific points in time. It also allows to check the state of each process or the global state for correctness at any time. If the error is found, the trace of system execution can be output for debugging. Thanks to deterministic simulation, the erroneous execution can be reliably reproduced until the error is fixed.

In addition to simulation-based testing, AnySystem supports _model checking_. This approach is based on  traversing the graph of possible system states, starting from a given initial state. A system state is determined by the states of processes, nodes and the network, and by the list of pending events. Arcs in the graph correspond to transitions between the states caused by events. Each system execution corresponds to a path in this graph. While simulation checks just a single execution path, model checking allows to explore all possible execution paths and exhaustively test the system. However, in practice this approach is often limited by a large size of system states graph. AnySystem allows to flexibly configure the model checking strategy to trade the exhaustiveness for reasonable execution time.

AnySystem is used in homework assignments for [Distributed Systems course](https://github.com/osukhoroslov/distsys-course-hse) at HSE University. In each assignment a student should implement a process logic for some system, e.g. a distributed key-value store, with given requirements. The correctness and other required properties of solutions are checked using the simulation-based and model checking tests.

## License

AnySystem is licensed under the [Apache-2.0 license](LICENSE-APACHE) or the [MIT license](LICENSE-MIT), at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in AnySystem by you, as defined in the Apache-2.0 license, shall be dual-licensed as above, without any additional terms or conditions.
