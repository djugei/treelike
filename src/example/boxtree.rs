#![cfg(std)]
use crate::Treelike;

/// A basic binary tree that stores its children in [Box]-es on the heap.
/// Used to show off trees that own the complete data and don't rely on any backing storage.
pub struct OwningBinaryTree<Content> {
	content: Content,
	children: [Option<Box<OwningBinaryTree<Content>>>; 2],
}

impl<Content: Default> Default for OwningBinaryTree<Content> {
	fn default() -> Self {
		OwningBinaryTree {
			content: Default::default(),
			children: [None, None],
		}
	}
}

use core::borrow::Borrow;
fn reborrow<A, R: Borrow<A>>(r: &R) -> &A { r.borrow() }

impl<'a, TreeCont> Treelike for &'a OwningBinaryTree<TreeCont> {
	type Content = &'a TreeCont;

	fn content(self) -> Self::Content { &self.content }

	type ChildIterator = core::iter::Map<
		core::iter::Flatten<core::slice::Iter<'a, Option<Box<OwningBinaryTree<TreeCont>>>>>,
		fn(&Box<OwningBinaryTree<TreeCont>>) -> &OwningBinaryTree<TreeCont>,
	>;

	fn children(self) -> Self::ChildIterator { self.children.into_iter().flatten().map(reborrow) }
}

#[test]
fn default_works() {
	let a: OwningBinaryTree<usize> = Default::default();

	a.first();
	a.last();
}

#[test]
fn option_box_size() {
	assert_eq!(
		std::mem::size_of::<Option<Box<usize>>>(),
		std::mem::size_of::<Box<usize>>()
	);
}
