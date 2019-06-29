use crate::Treelike;

/// A basic binary tree that contains references to its children
#[derive(Debug)]
pub struct BorrowingBinaryTree<'a, Content> {
	content: Content,
	children: [Option<&'a BorrowingBinaryTree<'a, Content>>; 2],
}

impl<Content: Default> Default for BorrowingBinaryTree<'_, Content> {
	fn default() -> Self {
		BorrowingBinaryTree {
			content: Default::default(),
			children: [None; 2],
		}
	}
}

impl<'a, TreeCont> Treelike for &'a BorrowingBinaryTree<'a, TreeCont> {
	type Content = &'a TreeCont;

	fn content(self) -> Self::Content { &self.content }

	type ChildIterator = core::iter::Cloned<
		core::iter::Flatten<core::slice::Iter<'a, Option<&'a BorrowingBinaryTree<'a, TreeCont>>>>,
	>;

	fn children(self) -> Self::ChildIterator { self.children.into_iter().flatten().cloned() }
}

#[test]
fn borrowing_tree_works() {
	let mut a: BorrowingBinaryTree<'_, usize> = Default::default();
	a.content = 0;

	let mut b: BorrowingBinaryTree<'_, usize> = Default::default();
	b.content = 1;

	let mut c: BorrowingBinaryTree<'_, usize> = Default::default();
	c.content = 2;

	let mut d: BorrowingBinaryTree<'_, usize> = Default::default();
	d.content = 3;

	c.children[0] = Some(&d);
	a.children[0] = Some(&b);
	a.children[1] = Some(&c);

	b.first();
	b.last();

	let mut state = Vec::new();
	a.callback_dft(|val, _depth| state.push(*val), ());
	assert_eq!(vec![1, 3, 2, 0], state);

	let mut limited = Vec::new();
	a.callback_dft(
		|val, _depth| limited.push(*val),
		(|_content, depth, _tree| depth == 0)
			as for<'r, 's> fn(&'r &usize, usize, &'s &BorrowingBinaryTree<'_, usize>) -> _,
	);
	assert_eq!(vec![1, 2, 0], limited);
}

#[test]
fn option_ref_size() {
	assert_eq!(
		std::mem::size_of::<Option<&usize>>(),
		std::mem::size_of::<&usize>()
	);
}
