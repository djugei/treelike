// basic design problem:
// either i ask for something Copy or at least Clone
// positive: people can pass in any kind of ro-ref,
// including their own types that encapsulate references, like LinTree
// drawback: mutability is off the table/requires locks
//
// or i don't put a clone restraint
// positive: enables mutability
// drawback: forces useless double-references cause it works with &Content and &Treelike, which is
// pointless if Content and Treelike are already &
// (possibly do some magic with AsRef/Borrow?)
//
// alternatively just have two traits
// positive: specialized to each use case (ro/rw)
// drawback: the two traits can't really be related, require user to implement twice
// also callback and filter are slightly incompatible due to taking references in case of mut
//

/// this trait is unfinished
pub trait TreelikeMut: Sized {
	type Content;

	type ChildIterator: Iterator<Item = Self>;
	fn into_tuple(self) -> (Self::Content, Self::ChildIterator);
	fn callback_dft_pre<CB: FnMut(&mut Self::Content, usize), F: FilterBuilderMut<Self>>(
		self,
		callback: CB,
		child_filter: F,
	) {
		callback_dft_pre(self, callback, child_filter, 0);
	}

	fn callback_dft<CB: FnMut(&mut Self::Content, usize), F: FilterBuilderMut<Self>>(
		self,
		callback: CB,
		child_filter: F,
	) {
		callback_dft(self, callback, child_filter, 0);
	}
}

fn callback_dft_pre<T: TreelikeMut, CB: FnMut(&mut T::Content, usize), F: FilterBuilderMut<T>>(
	t: T,
	mut cb: CB,
	mut f: F,
	depth: usize,
) -> (CB, F) {
	let (mut content, children) = t.into_tuple();
	cb(&mut content, depth);

	let filter = f.build(&content, depth, children);
	for child in filter {
		let (i_cb, i_f) = callback_dft_pre(child, cb, f, depth + 1);
		cb = i_cb;
		f = i_f;
	}

	(cb, f)
}

fn callback_dft<T: TreelikeMut, CB: FnMut(&mut T::Content, usize), F: FilterBuilderMut<T>>(
	t: T,
	mut cb: CB,
	mut f: F,
	depth: usize,
) -> (CB, F) {
	let (mut content, children) = t.into_tuple();

	let filter = f.build(&content, depth, children);
	for child in filter {
		let (i_cb, i_f) = callback_dft(child, cb, f, depth + 1);
		cb = i_cb;
		f = i_f;
	}

	cb(&mut content, depth);

	(cb, f)
}

pub trait FilterBuilderMut<T: TreelikeMut> {
	type Filter: Iterator<Item = T>;
	fn build(&self, content: &T::Content, depth: usize, children: T::ChildIterator)
		-> Self::Filter;
}

// change cb to take &Content, mut trees can then make content be &mut realcontent
// change filter to take &self and &content
// filter can probably require copy or at least clone?
