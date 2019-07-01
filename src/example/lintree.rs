use crate::Treelike;

/// A tree whose nodes are stored in a backing slice.
///
/// The root node is at index 0 and the child
/// nodes at `index*2 + 1` and `index*2 + 2`.
///
/// Used in most examples, as it is easy to initialize.
///
/// Also shows a case where Treelike is implemented on the node itself, and not a reference to one.
/// An important pitfall for that is that you may need to manually implement [Copy] and [Clone] as
/// deriving them places [Copy]/[Clone] bounds on all type parameters (`T` in this case), even
/// though that might not be necessary due to the content not being stored in-line, and therefore
/// not being [Copy]/[Clone]d.
#[derive(Debug)]
pub struct LinTree<'a, T> {
	index: usize,
	slice: &'a [T],
}

impl<'a, T> LinTree<'a, T> {
	pub fn new(index: usize, slice: &'a [T]) -> Self { LinTree { index, slice } }

	fn tuple_new((index, slice): (usize, &'a [T])) -> Option<Self> {
		slice.get(index).map(|_| Self::new(index, slice))
	}
}

impl<'a, T> Copy for LinTree<'a, T> {}

impl<'a, T> Clone for LinTree<'a, T> {
	fn clone(&self) -> Self {
		Self {
			index: self.index,
			slice: self.slice,
		}
	}
}

impl<'a, T: core::fmt::Debug> Treelike for LinTree<'a, T> {
	type Content = &'a T;

	type ChildIterator = core::iter::FlatMap<
		core::iter::Zip<
			core::iter::Chain<core::iter::Once<usize>, core::iter::Once<usize>>,
			core::iter::Repeat<&'a [T]>,
		>,
		Option<LinTree<'a, T>>,
		fn((usize, &'a [T])) -> Option<LinTree<'a, T>>,
	>;

	fn content(self) -> Self::Content { &self.slice[self.index] }

	fn children(self) -> Self::ChildIterator {
		use core::iter::{once, repeat};
		let left = 2 * self.index + 1;
		let right = 2 * self.index + 2;
		once(left)
			.chain(once(right))
			.zip(repeat(self.slice))
			.flat_map(Self::tuple_new)
	}

	/// This is also an example of overriding the [Treelike]s default implementations where
	/// necessary. LinTree can provide breadth-first traversal with a simple iteration
	fn callback_bft<CB: FnMut(Self::Content, usize)>(self, mut callback: CB) {
		let usize_bits = (core::mem::size_of::<usize>() * 8) as u32;
		for (i, content) in self.slice.iter().enumerate().skip(self.index) {
			// this is flooring log2 for integers
			// find the first one by subtracting the bit-length from the leading_zeros
			// first one - 1 is already floored log2
			let depth = usize_bits - (i + 1).leading_zeros() - 1;
			callback(content, depth as usize);
		}
	}

	//TODO could also implement filtered bft, but that requires itertools group_by. put that in
	//as an optional dependency maybe
}

#[test]
fn depth_test() {
	let base = [0, (1), 2, (3), 4, 5, 6, (7), 8, 9, 10, 11, 12, 13, 14, (15)];

	let root = LinTree::new(0, &base);
	let mut state = Vec::new();
	root.callback_bft(|_content, depth| state.push(depth));
	assert_eq!(&state, &[0, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 3, 3, 3, 3, 4]);

	let deeper = LinTree::new(4, &base);
	let mut state = Vec::new();
	deeper.callback_bft(|_content, depth| state.push(depth));
	assert_eq!(&state, &[2, 2, 2, 3, 3, 3, 3, 3, 3, 3, 3, 4]);
}

#[test]
fn basic_test() {
	let base = [3, 4, 5, 6, 7];
	let root = LinTree::new(0, &base);

	let mut state = Vec::new();
	root.callback_dft(|val, _depth| state.push(*val), ());
	assert_eq!(vec![6, 7, 4, 5, 3], state);
}

#[test]
fn iter_test() {
	let base = [0, (1), 2, (3), 4, 5, 6, (7), 8, 9, 10, 11, 12, 13, 14, (15)];
	let root = LinTree::new(0, &base);

	let mut state = Vec::new();
	root.callback_dft(|val, _depth| state.push(*val), ());

	let iter_state: Vec<_> = root.iter_dft(()).cloned().collect();
	assert_eq!(iter_state, state);
}
