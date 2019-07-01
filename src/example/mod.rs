//! This module contains some basic example trees that are used both for tests and for you to get
//! some inspiration.

mod borrowtree;
mod lintree;

pub use borrowtree::BorrowingBinaryTree;
pub use lintree::LinTree;

#[cfg(feature = "std")]
mod boxtree;
#[cfg(feature = "std")]
pub use boxtree::OwningBinaryTree;
