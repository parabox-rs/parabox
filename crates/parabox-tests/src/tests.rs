use crate::logger::init_logger;
use parabox_macros::scan_tests;
use parabox_parser::{Executor, NamedStringSource, ParseResult};
use std::rc::Rc;

#[scan_tests("crates/parabox-tests/worlds/")]
#[test]
fn run_test(name: &str, text: &str) {
    let _guard = init_logger();

    if let Err(e) = execute_with_debug_info(name, text) {
        eprintln!("{}", e);
        panic!("{}", e.message());
    }
}

fn execute_with_debug_info(name: &str, text: &str) -> ParseResult<()> {
    let mut executor = Executor::new();
    let source = NamedStringSource::new(name.into(), text.to_string());
    executor.push_source(Rc::new(source))?;

    let mut traces = vec![];
    let mut last_format = None;

    while executor.has_next() {
        let span = executor.step().map_err(|e| {
            for output in &traces {
                println!("{}", output);
            }
            e
        })?;

        let pushing = span.text()[..4].to_lowercase() == "push";

        let current = if !pushing {
            center("Initial".to_string(), 24)
        } else {
            center(format!("Line {}", span.locate().0 + 1), 24)
                + "\n"
                + format!(">>> {}", span.text()).as_str()
        } + "\n"
            + executor.format_positions().as_str();

        if pushing {
            if let Some(last) = last_format.take() {
                traces.push(last)
            }
            traces.push(current)
        } else {
            last_format = Some(current);
        }
    }

    for output in traces {
        println!("{}", output);
    }

    Ok(())
}

fn center(s: String, width: usize) -> String {
    let len = s.len();
    if len >= width {
        return s;
    }

    let left = (width - len) / 2 - 1;
    let right = width - len - left - 1;

    format!("{} {} {}", "=".repeat(left), s, "=".repeat(right))
}
