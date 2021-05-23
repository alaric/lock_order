//! # Lock ordering macro.
//!
//! ## Raison D'Être
//!
//! This crate provides a simple lock ordering procmacro for ensuring a deterministic locking order,
//! which is useful as a pattern to prevent deadlocks between fine-grain mutex use.
//!
//! It also serves to remove the `unwrap()` of panic-propagation between threads in the case of
//! poisoned locks. This is my favoured approach for handling an already panicking program, but
//! makes it difficult to find other non-valid usages of `unwrap()` in the code.
//!
//! ## Basic Usage
//!
//! - The `mut` is optional based on if you want mutability, but must be prior to the identifier
//! - The identifier can be multiple field lookups, ie `self.locks.connections` and will result in a
//! bound variable `connections` as the last part of the full identifier.
//! - There can be one or more locks provided, separated by `,`, they will be ordered
//! lexicographially by the bound variable name.
//!
//! Thus an example like this:
//! ```
//! use lock_order::lock;
//! use std::sync::Mutex;
//!
//! let lock1 = Mutex::new(1);
//! let lock2 = Mutex::new(2);
//! let lock3 = Mutex::new(3);
//! {
//!     lock!(mut lock2, lock3, mut lock1);
//!     *lock1 = 3 + *lock3;
//!     *lock2 = 4 + *lock3;
//! }
//! ```
//!
//! Would expand to:
//!
//! ```
//! use lock_order::lock;
//! use std::sync::Mutex;
//!
//! let lock1 = Mutex::new(1);
//! let lock2 = Mutex::new(2);
//! let lock3 = Mutex::new(3);
//! {
//!     let (mut lock1, mut lock2, lock3) = (lock1.lock().unwrap(), lock2.lock().unwrap(),
//!     lock3.lock().unwrap());
//!     *lock1 = 3 + *lock3;
//!     *lock2 = 4 + *lock3;
//! }
//! ```
//!

//! ## Future direction
//!
//! - Support for RwLock
//! - Support for bare non-poisoning locks such as `parking_lot`, which don't require `unwrap()`.

use proc_macro::{self, TokenStream};

#[derive(Clone, PartialEq, Debug, Default)]
struct LockItem {
    last_identifier: String,
    full_identifier: String,
    mutable: bool,
}

impl LockItem {
    fn add(&mut self, id: &proc_macro::TokenTree) {
        self.full_identifier += &id.to_string();
        self.last_identifier = id.to_string();
    }
}

/// Lock one or more locks at a time.
///
/// This takes multiple lock arguments (with an optional `mut` flag) and creates a single let
/// expression binding the `.lock().unwrap()` into variables of the same name as the last identifier
/// in the lock expression. This means that if something is passed such as:
///
/// ```
/// # use lock_order::lock;
/// # use std::sync::Mutex;
/// # struct Inner {
/// #    connections: Mutex<u32>,
/// # }
/// # struct Test {
/// #    locks: Inner,
/// # }
/// # impl Test {
/// # fn test(&self) {
/// lock!(mut self.locks.connections);
/// # }
/// # }
/// ```
///
/// Then the output will be something similar to:
///
/// ```
/// # use lock_order::lock;
/// # use std::sync::Mutex;
/// # struct Inner {
/// #    connections: Mutex<u32>,
/// # }
/// # struct Test {
/// #     locks: Inner,
/// # }
/// # impl Test {
/// # fn test(&self) {
/// let (mut connection) = (self.locks.connections.lock().unwrap());
/// # }
/// # }
/// ```
#[proc_macro]
pub fn lock(item: TokenStream) -> TokenStream {
    let mut out = Vec::new();
    let mut curr = LockItem::default();
    for i in item {
        // FIX this should probably not be just operating on strings
        match i.to_string().as_str() {
            "mut" => {
                curr.mutable = true;
            }
            "," => {
                out.push(curr);
                curr = LockItem::default();
            }
            _ => {
                curr.add(&i);
            }
        }
    }

    if curr != LockItem::default() {
        out.push(curr);
    }

    out.sort_by(|a, b| a.last_identifier.partial_cmp(&b.last_identifier).unwrap());

    let declarations: Vec<String> = out
        .clone()
        .into_iter()
        .map(|x| {
            if x.mutable {
                format!("mut {}", x.last_identifier)
            } else {
                x.last_identifier.clone()
            }
        })
        .collect();
    let locks: Vec<String> = out
        .into_iter()
        .map(|x| format!("{}.lock().unwrap()", x.full_identifier))
        .collect();

    format!(
        "let ({}) = ({});",
        declarations.join(", "),
        locks.join(", "),
    )
    .parse()
    .unwrap()
}
