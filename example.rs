
//! Module-level doc comment (`@comment`)
// Single-line comment (`@comment`)
/* Multi-line
   block comment (`@comment`) */

#![allow(dead_code)] // Inner attribute (`@attribute`, `@punctuation.special`)

use std::collections::HashMap as Map; // `@keyword`, `@type`, `@operator`
use std::fmt::{Debug, Display};

/// Const and static items (`@constant`)
pub const MAX_RETRIES: usize = 3;
static GLOBAL_FLAG: bool = true;

/// Type alias and lifetimes (`@type`, `@label` / `@storageclass`)
pub type Result<T> = std::result::Result<T, &'static str>;

/// Enum with variants (`@type`, `@constructor` / `@constant`)
#[derive(Debug, Clone, PartialEq)]
pub enum Status<'a> {
    Active,
    Pending(u64),
    Failed { reason: &'a str, code: i32 },
}

/// Struct with fields (`@type`, `@property`)
pub struct Worker<'a, T: Display> {
    pub id: u32,
    label: &'a str,
    payload: T,
}

/// Trait definition (`@type`, `@function`, `@variable.parameter`)
pub trait Runnable {
    type Output;
    fn run(&mut self, dry_run: bool) -> Self::Output;

    fn default_hook(&self) {
        println!("Running default hook"); // `@function.macro`
    }
}

/// Inherent and trait implementations (`@keyword`, `@type`, `@variable.builtin`)
impl<'a, T: Display> Worker<'a, T> {
    pub fn new(id: u32, label: &'a str, payload: T) -> Self {
        Self { id, label, payload }
    }

    pub async fn process(&self, items: &[i32]) -> Option<i32> {
        let mut total = 0_i32;

        // Loops, labels, ranges, and operators (`@label`, `@keyword`, `@operator`)
        'outer: for (idx, &val) in items.iter().enumerate() {
            if val < 0 {
                continue 'outer;
            } else if val > 1_000 {
                break;
            }
            total += val * (idx as i32);
        }

        // Pattern matching and enum constructors
        let status = Status::Pending(42);
        match status {
            Status::Active => Some(total),
            Status::Pending(n) if n > 10 => Some(total + n as i32),
            Status::Failed { reason, .. } => {
                eprintln!("Error: {}", reason);
                None
            }
            _ => None,
        }
    }
}

impl<'a, T: Display> Runnable for Worker<'a, T> {
    type Output = String;

    fn run(&mut self, dry_run: bool) -> Self::Output {
        if dry_run {
            return String::from("dry-run");
        }
        format!("{}: {}", self.label, self.payload)
    }
}

/// Declarative macro definition (`@keyword`, `@function.macro`)
macro_rules! make_map {
    ( $( $k:expr => $v:expr ),* $(,)? ) => {{
        let mut m = Map::new();
        $( m.insert($k, $v); )*
        m
    }};
}

/// Literals, escape sequences, raw strings, closures, and unsafe (`@string`, `@escape`, `@constant.builtin`)
pub fn showcase_syntax() {
    let hex = 0xff_u8;
    let float = 3.14_f64;
    let ch = '\n';
    let byte_str = b"raw bytes\x00";
    let escaped = "Line 1\n\t\"Quoted\" \\ backslash";
    let raw = r#"Raw string with "quotes" and no escapes"#;

    let closure = |x: i32, y: i32| -> i32 { x ^ y | (x & y) << 1 };
    let map = make_map!["a" => 1, "b" => 2];

    unsafe {
        let ptr: *const u8 = &hex;
        let _val = *ptr;
    }
    println!("hello world");
    let _ = (float, ch, byte_str, escaped, raw, closure(3, 5), map);
}