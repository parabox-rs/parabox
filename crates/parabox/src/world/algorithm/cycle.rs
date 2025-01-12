use std::fmt::Debug;

/// The initial capacity of the trace.
const TRACE_INIT_CAP: usize = 20;
/// The maximum length of the trace, `0` means no limit.
const TRACE_BURN: usize = 1000;

/// A cycle resolver.
///
/// It is used to detect cycles in a queue.
///
/// Each time [Cycle::push] is called, it will check if the key is already in
/// the trace. Users can then deal with the duplicate nodes and try to push
/// again.
///
/// It will also limits the trace length to [TRACE_BURN] to avoid potential
/// infinite loops (which theoerically won't happen, but are still possible if
/// there is a bug).
pub(crate) struct Cycle<T, V = ()> {
    trace: Vec<(T, V)>,
}

impl<T, V> Cycle<T, V> {
    /// Create a new cycle resolver.
    ///
    /// The trace is initialized with a capacity of [TRACE_INIT_CAP].
    pub fn new() -> Self {
        Self {
            trace: Vec::with_capacity(TRACE_INIT_CAP),
        }
    }

    pub fn pop(&mut self) -> Option<(T, V)> {
        self.trace.pop()
    }

    pub fn is_empty(&self) -> bool {
        self.trace.is_empty()
    }
}

impl<T: Eq, V> Cycle<T, V> {
    /// Panics if the cycle burns.
    pub fn push(&mut self, key: T, value: V) -> Option<&V> {
        if let Some(index) = self.trace.iter().position(|x| &x.0 == &key) {
            return Some(&self.trace[index].1);
        }

        self.trace.push((key, value));

        if TRACE_BURN > 0 && self.trace.len() > TRACE_BURN {
            panic!("Cycle burns.");
        }

        None
    }
}

impl<T: Debug, V: Debug> Debug for Cycle<T, V> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for item in &self.trace {
            write!(f, "{:?} -> ", item)?;
        }
        write!(f, "...")
    }
}
