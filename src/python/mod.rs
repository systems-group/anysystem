//! Python integration.

use std::ffi::CString;
use std::fs;
use std::rc::Rc;

use colored::Colorize;
use pyo3::call::PyCallArgs;
use pyo3::prelude::*;
use pyo3::types::{PyModule, PyString};

use crate::process::StringProcessState;
use crate::{Context, Message, Process, ProcessState};

#[cfg(test)]
mod tests;

/// Creates instances of [`PyProcess`].
pub struct PyProcessFactory {
    proc_class: Py<PyAny>,
    msg_class: Rc<Py<PyAny>>,
    ctx_class: Rc<Py<PyAny>>,
    get_size_fun: Rc<Py<PyAny>>,
}

impl PyProcessFactory {
    /// Creates a process factory using the specified Python file and class name.
    pub fn new(impl_path: &str, impl_class: &str) -> Self {
        let impl_code = CString::new(fs::read_to_string(impl_path).unwrap()).unwrap();
        let impl_realpath = fs::canonicalize(impl_path).unwrap();
        let impl_filename = impl_realpath.to_str().unwrap();
        let impl_module = CString::new(impl_filename.replace(".py", "")).unwrap();
        let impl_filename = CString::new(impl_filename).unwrap();
        let classes = Python::attach(|py| -> (Py<PyAny>, Py<PyAny>, Py<PyAny>, Py<PyAny>) {
            let impl_module = PyModule::from_code(py, &impl_code, &impl_filename, &impl_module).unwrap();
            let proc_class = impl_module.getattr(impl_class).unwrap().into();
            let msg_class = impl_module.getattr("Message").unwrap().into();
            let ctx_class = impl_module.getattr("Context").unwrap().into();
            let get_size_fun = get_size_fun(py);
            (proc_class, msg_class, ctx_class, get_size_fun)
        });
        Self {
            proc_class: classes.0,
            msg_class: Rc::new(classes.1),
            ctx_class: Rc::new(classes.2),
            get_size_fun: Rc::new(classes.3),
        }
    }

    /// Creates a process instance with specified arguments and random seed.
    pub fn build(&self, args: impl for<'py> PyCallArgs<'py>, seed: u64) -> PyProcess {
        let proc = Python::attach(|py| -> Py<PyAny> {
            let code = CString::new(format!("import random\nrandom.seed({seed})")).unwrap();
            py.run(&code, None, None).unwrap();
            self.proc_class
                .call1(py, args)
                .map_err(|e| {
                    eprintln!("{}\n", "!!! Error when creating Python process:".red());
                    eprintln!("{}", error_to_string(e, py).red());
                    "Error when creating Python process"
                })
                .unwrap()
        });
        PyProcess {
            proc,
            msg_class: self.msg_class.clone(),
            ctx_class: self.ctx_class.clone(),
            get_size_fun: self.get_size_fun.clone(),
            max_size: 0,
            max_size_freq: 0,
            max_size_counter: 0,
        }
    }
}

/// Process implementation backed by a Python object.
pub struct PyProcess {
    proc: Py<PyAny>,
    msg_class: Rc<Py<PyAny>>,
    ctx_class: Rc<Py<PyAny>>,
    get_size_fun: Rc<Py<PyAny>>,
    max_size: u64,
    max_size_freq: u32,
    max_size_counter: u32,
}

impl PyProcess {
    /// Sets the frequency of updating the maximum size of process inner data.
    pub fn set_max_size_freq(&mut self, freq: u32) {
        self.max_size_freq = freq;
        self.max_size_counter = 1;
    }

    fn handle_proc_actions(ctx: &mut Context, py_ctx: &Py<PyAny>, py: Python) {
        let sent: Vec<(String, String, String)> = py_ctx.getattr(py, "_sent_messages").unwrap().extract(py).unwrap();
        for m in sent {
            ctx.send(Message::new(&m.0, &m.1), m.2);
        }
        let sent_local: Vec<(String, String)> =
            py_ctx.getattr(py, "_sent_local_messages").unwrap().extract(py).unwrap();
        for m in sent_local {
            ctx.send_local(Message::new(&m.0, &m.1));
        }
        let timer_actions: Vec<(String, f64, bool)> =
            py_ctx.getattr(py, "_timer_actions").unwrap().extract(py).unwrap();
        for t in timer_actions {
            if t.1 < 0.0 {
                ctx.cancel_timer(&t.0);
            } else if t.2 {
                ctx.set_timer_once(&t.0, t.1);
            } else {
                ctx.set_timer(&t.0, t.1);
            }
        }
    }

    fn update_max_size(&mut self, py: Python, force_update: bool) {
        if self.max_size_freq > 0 {
            self.max_size_counter -= 1;
            if self.max_size_counter == 0 || force_update {
                let size: u64 = self.get_size_fun.call1(py, (&self.proc,)).unwrap().extract(py).unwrap();
                self.max_size = self.max_size.max(size);
                self.max_size_counter = self.max_size_freq;
            }
        }
    }

    fn clone_process(&self) -> Py<PyAny> {
        Python::attach(|py| -> Py<PyAny> {
            let module = PyModule::import(py, "copy").unwrap();
            let fun = module.getattr("deepcopy").unwrap();
            fun.call1((&self.proc,)).unwrap().into()
        })
    }
}

impl Process for PyProcess {
    fn on_message(&mut self, msg: Message, from: String, ctx: &mut Context) -> Result<(), String> {
        Python::attach(|py| {
            let py_msg = self
                .msg_class
                .call_method1(py, "from_json", (msg.tip, msg.data))
                .unwrap();
            let py_ctx = self.ctx_class.call1(py, (ctx.time(),)).unwrap();
            self.proc
                .call_method1(py, "on_message", (py_msg, from, &py_ctx))
                .map_err(|e| error_to_string(e, py))?;
            PyProcess::handle_proc_actions(ctx, &py_ctx, py);
            self.update_max_size(py, false);
            Ok(())
        })
    }

    fn on_local_message(&mut self, msg: Message, ctx: &mut Context) -> Result<(), String> {
        Python::attach(|py| {
            let py_msg = self
                .msg_class
                .call_method1(py, "from_json", (msg.tip, msg.data))
                .unwrap();
            let py_ctx = self.ctx_class.call1(py, (ctx.time(),)).unwrap();
            self.proc
                .call_method1(py, "on_local_message", (py_msg, &py_ctx))
                .map_err(|e| error_to_string(e, py))?;
            PyProcess::handle_proc_actions(ctx, &py_ctx, py);
            self.update_max_size(py, false);
            Ok(())
        })
    }

    fn on_timer(&mut self, timer: String, ctx: &mut Context) -> Result<(), String> {
        Python::attach(|py| {
            let py_ctx = self.ctx_class.call1(py, (ctx.time(),)).unwrap();
            self.proc
                .call_method1(py, "on_timer", (timer, &py_ctx))
                .map_err(|e| error_to_string(e, py))?;
            PyProcess::handle_proc_actions(ctx, &py_ctx, py);
            self.update_max_size(py, false);
            Ok(())
        })
    }

    fn max_size(&mut self) -> u64 {
        Python::attach(|py| self.update_max_size(py, true));
        self.max_size
    }

    fn state(&self) -> Result<Rc<dyn ProcessState>, String> {
        Python::attach(|py| -> Result<Rc<dyn ProcessState>, String> {
            let res = self
                .proc
                .call_method0(py, "get_state")
                .map_err(|e| error_to_string(e, py))?;
            Ok(Rc::new(res.as_ref().cast_bound::<PyString>(py).unwrap().to_string()))
        })
    }

    fn set_state(&mut self, state: Rc<dyn ProcessState>) -> Result<(), String> {
        let data = state.downcast_rc::<StringProcessState>().unwrap();
        Python::attach(|py| {
            self.proc
                .call_method1(py, "set_state", ((*data).clone(),))
                .map_err(|e| error_to_string(e, py))?;
            Ok(())
        })
    }
}

impl Clone for PyProcess {
    fn clone(&self) -> Self {
        PyProcess {
            // process is cloned via deepcopy to avoid leaking state between MC and simulation
            proc: self.clone_process(),
            msg_class: self.msg_class.clone(),
            ctx_class: self.ctx_class.clone(),
            get_size_fun: self.get_size_fun.clone(),
            max_size: self.max_size,
            max_size_freq: self.max_size_freq,
            max_size_counter: self.max_size_counter,
        }
    }
}

fn get_size_fun(py: Python) -> Py<PyAny> {
    PyModule::from_code(
        py,
        &CString::new("
# Adapted from https://github.com/bosswissam/pysize
import sys
import inspect

def get_size(obj, seen=None):
    size = sys.getsizeof(obj)
    if seen is None:
        seen = set()
    obj_id = id(obj)
    if obj_id in seen:
        return 0
    # Important mark as seen *before* entering recursion to gracefully handle
    # self-referential objects
    seen.add(obj_id)
    if hasattr(obj, '__dict__'):
        for cls in obj.__class__.__mro__:
            if '__dict__' in cls.__dict__:
                d = cls.__dict__['__dict__']
                if inspect.isgetsetdescriptor(d) or inspect.ismemberdescriptor(d):
                    size += get_size(obj.__dict__, seen)
                break
    if isinstance(obj, dict):
        size += sum((get_size(v, seen) for v in obj.values()))
        size += sum((get_size(k, seen) for k in obj.keys()))
    elif hasattr(obj, '__iter__') and not isinstance(obj, (str, bytes, bytearray)):
        try:
            size += sum((get_size(i, seen) for i in obj))
        except TypeError:
            raise Exception(\"Unable to get size of %r. This may lead to incorrect sizes. Please report this error.\", obj)
    if hasattr(obj, '__slots__'): # can have __slots__ with __dict__
        size += sum(get_size(getattr(obj, s), seen) for s in obj.__slots__ if hasattr(obj, s))
    return size").unwrap(),
        &CString::new("").unwrap(),
        &CString::new("").unwrap(),
    )
    .unwrap()
    .getattr("get_size")
    .unwrap()
    .into()
}

fn error_to_string(err: PyErr, py: Python) -> String {
    err.to_string() + "\n" + &err.traceback(py).unwrap().format().unwrap()
}
