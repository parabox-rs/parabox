//! The parser and the executor for the Parabox script.
//!
//! # Overview
//!
//! The parser can parse the Parabox script and executes them in a Parabox
//! world. Each line of the script is a command, which
//! - _Define_ a block of a prototype.
//! - _Place_ a block at a position.
//! - _Push_ a block in a direction.
//! - _Expect_ a block at a position.
//!
//! See crate [`parabox`] for more information about the Parabox world.
//!
//! [`parabox`]: https://crates.io/crates/parabox
//!
//! # Syntax
//!
//! A command is made up of _keywords_, _identifiers_ and _size tuples_:
//! - _keywords_ are case-insensitive and are made up of alphabetic characters.
//! -_identifiers_ are case-sensitive and are started by a `#` character
//! followed by a series of alphabetic, numeric, or underscore characters.
//! - _size tuples_ are of the form `(<x>, <y>)` where `<x>` and `<y>` are
//! unsigned integers.
//!
//! These different parts are separated by spaces.
//!
//! You can also write inline comments by starting with `//`.
//!
//! ## Define
//!
//! ```text
//! DEFINE <prototype> <identifier> [SIZE (<width>, <height>)] [REF <reference>] [SOLID]
//! ```
//!
//! The different properties are interchangeable, and each prototype has its own
//! allowed and required properties.
//!
//! The `SIZE` property and the `REF` property are the same as those defined in
//! [`MetaProtoType`]. See the enum [`MetaProtoType`] for more information.
//!
//! The `SOLID` property means defining a block with `SIZE (1, 1)` and place a
//! wall inside it. It can only be used on box prototype, and will conflict with
//! `SIZE` property.
//!
//! [`MetaProtoType`]: crate::MetaProtoType
//!
//! ## Place
//!
//! ```text
//! PLACE <identifier> [AT (<x>, <y>)] [IN <container>] [ORPHAN]
//! ```
//!
//! Again, the properties are interchangable.
//!
//! You should either specify both `AT` and `IN` property, or specify `ORPHAN`.
//!
//! Usually you don't need to specify `ORPHAN` property, since the default
//! position of a block is orphan.
//!
//! ## Push
//!
//! ```text
//! PUSH <identifier> [NORTH | SOUTH | EAST | WEST] [[MOVED | STATIC]]
//! ```
//!
//! The `MOVED` and `STATIC` properties are optional. If not specified, no
//! assertion is made. See the enum [`Assertion`] for more information.
//!
//! [`Assertion`]: crate::Assertion
//!
//! ## Expect
//!
//! ```text
//! EXPECT <identifier> [AT (<x>, <y>)] [IN <container>] [ORPHAN]
//! ```
//!
//! The properties are the same as those in the `PLACE` command.
//!
//! # Execution
//!
//! Use [`Executor`] to execute commands. If you want to parse a script only,
//! use [`parse`] function. See [`Executor`] and [`parse`] for more information.
//!
//! [`Executor`]: crate::Executor
//! [`parse`]: crate::parse
//!
//! # Examples
//!
//! ```
//! # use std::rc::Rc;
//! # use parabox_parser::{Executor, StringSource};
//!
//! let script = r#"
//! DEFINE BOX #container size (3, 3)
//! DEFINE BOX #box1 solid
//! DEFINE BOX #box2 size (3, 3)
//! DEFINE WALL #wall
//!
//! PLACE #box1 at (0, 1) in #container
//! PLACE #box2 at (1, 1) in #container
//! PLACE #wall at (2, 1) in #container
//!
//! PUSH #box1 east MOVED
//!
//! EXPECT #box1 at (0, 1) in #box2
//! EXPECT #box2 at (1, 1) in #container
//! "#;
//!
//! let source = Rc::new(StringSource::new(script.to_string()));
//! let mut executor = Executor::new();
//!
//! executor.push_source(source).unwrap();
//! executor.run_all().unwrap();
//!
//! println!("{}", executor.format_positions());
//!
//! let (_world, meta) = executor.take();
//!
//! println!("{:?}", meta);
//! ```

#![allow(dead_code)]
#![warn(missing_docs)]

extern crate self as parabox_parser;

mod command;
mod error;
mod executor;
mod kind;
mod lexer;
mod meta;
mod parser;
mod source;

pub use command::{Assertion, Command, MetaPosition, MetaProtoType, Operation};
pub use error::{ParseError, ParseResult, Span};
pub use executor::Executor;
pub use meta::{MetaKey, MetaName, MetaTable};
pub use parser::{parse, SpannedCommand};
pub use source::{FileSource, NamedStringSource, Source, StringSource};

#[cfg(test)]
mod tests {
    use crate::command::{Assertion, Command, MetaProtoType};
    use crate::parser::{parse, SpannedCommand};
    use crate::source::StringSource;
    use parabox::Direction;
    use std::rc::Rc;

    fn parse_command(text: &str) -> Result<Vec<SpannedCommand>, String> {
        parse(Rc::new(StringSource::new(text.to_string()))).map_err(|e| e.to_string())
    }

    #[test]
    fn test_comment() {
        let result = parse_command("// this is a comment").unwrap();

        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_define_wall() {
        let result = parse_command("define wall #wall").unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(
            result[0].command(),
            &Command::define("wall".into(), MetaProtoType::Wall)
        );
    }

    #[test]
    fn test_define_box() {
        let result = parse_command("define box #box size (1, 1)").unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(
            result[0].command(),
            &Command::define("box".into(), MetaProtoType::Box { size: (1, 1) })
        );
    }

    #[test]
    fn test_define_alias() {
        let result = parse_command("define alias #alias ref #box").unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(
            result[0].command(),
            &Command::define(
                "alias".into(),
                MetaProtoType::Alias {
                    reference: "box".into()
                }
            )
        );
    }

    #[test]
    fn test_define_infinity() {
        let result = parse_command("define infinity #infinity ref #box").unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(
            result[0].command(),
            &Command::define(
                "infinity".into(),
                MetaProtoType::Infinity {
                    reference: "box".into()
                }
            )
        );
    }

    #[test]
    fn test_define_epsilon() {
        let result = parse_command("define epsilon #epsilon ref #box size (1, 1)").unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(
            result[0].command(),
            &Command::define(
                "epsilon".into(),
                MetaProtoType::Epsilon {
                    reference: "box".into(),
                    size: (1, 1)
                }
            )
        );
    }

    #[test]
    fn test_define_void() {
        let result = parse_command("define void #void size (1, 1)").unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(
            result[0].command(),
            &Command::define("void".into(), MetaProtoType::Void { size: (1, 1) })
        );
    }

    #[test]
    fn test_define_solid() {
        let result = parse_command("define box #solid solid").unwrap();

        assert_eq!(result.len(), 3);
        assert_eq!(
            result[0].command(),
            &Command::define("solid".into(), MetaProtoType::Box { size: (1, 1) })
        );
        assert_eq!(
            result[1].command(),
            &Command::define("solid::interior".into(), MetaProtoType::Wall)
        );
        assert_eq!(
            result[2].command(),
            &Command::place("solid::interior".into(), Some("solid".into()), (0, 0))
        );
    }

    #[test]
    fn test_place_in_container() {
        let result = parse_command("place #box at (1, 1) in #container").unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(
            result[0].command(),
            &Command::place("box".into(), Some("container".into()), (1, 1))
        );
    }

    #[test]
    fn test_place_orphan() {
        let result = parse_command("place #box orphan").unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(
            result[0].command(),
            &Command::place("box".into(), None, (0, 0))
        );
    }

    #[test]
    fn test_push() {
        let result = parse_command("push #box east").unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(
            result[0].command(),
            &Command::push("box".into(), Direction::East, Assertion::None)
        );
    }

    #[test]
    fn test_push_assert_moved() {
        let result = parse_command("push #box east moved").unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(
            result[0].command(),
            &Command::push("box".into(), Direction::East, Assertion::Moved)
        );
    }

    #[test]
    fn test_push_assert_static() {
        let result = parse_command("push #box east static").unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(
            result[0].command(),
            &Command::push("box".into(), Direction::East, Assertion::Static)
        );
    }

    #[test]
    fn test_expect() {
        let result = parse_command("expect #box at (1, 1) in #container").unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(
            result[0].command(),
            &Command::expect("box".into(), Some("container".into()), (1, 1))
        );
    }

    #[test]
    fn test_fail_on_invalid_syntax() {
        let commands = vec![
            "invalid statement",
            "define invalid proto",
            "define wall #invalid-ident",
            "define wall #wall invalid property",
        ];

        for command in commands {
            assert!(parse_command(command).is_err());
        }
    }

    #[test]
    fn test_fail_on_define_with_unexpected_size() {
        for proto in &["wall", "alias", "infinity"] {
            assert!(parse_command(&format!("define {} #{} size (1, 1)", proto, proto)).is_err());
        }
    }

    #[test]
    fn test_fail_on_define_with_unexpected_ref() {
        for proto in &["wall", "box", "void"] {
            assert!(parse_command(&format!("define {} #{} ref #box", proto, proto)).is_err());
        }
    }

    #[test]
    fn test_fail_on_define_with_unexpected_solid() {
        for proto in &["wall", "alias", "infinity", "epsilon", "void"] {
            assert!(parse_command(&format!("define {} #{} solid", proto, proto)).is_err());
        }
    }

    #[test]
    fn test_fail_on_place_with_not_enough_arguments() {
        let commands = vec![
            "place #box",
            "place #box in #container",
            "place #box at (2, 2)",
        ];

        for command in commands {
            assert!(parse_command(command).is_err());
        }
    }

    #[test]
    fn test_fail_on_push_with_not_enough_arguments() {
        let commands = vec!["push #box"];

        for command in commands {
            assert!(parse_command(command).is_err());
        }
    }

    #[test]
    fn test_fail_on_expect_with_not_enough_arguments() {
        let commands = vec![
            "expect #box",
            "expect #box in #container",
            "expect #box at (2, 2)",
        ];

        for command in commands {
            assert!(parse_command(command).is_err());
        }
    }
}
