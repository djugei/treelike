#[cfg(feature = "alloc")]
use alloc::collections::VecDeque;
#[cfg(feature = "alloc")]
use alloc::vec::Vec;

/// The main Trait of the crate.
/// Provides many kinds of iterations and searches on trees.
///
/// Just like [Iterator] most functions are provided,
/// only [children][Treelike::children] and [content][Treelike::content] need to be implemented.
///
/// Should probably be implemented on references of the node-type,
/// unless your node itself is already [Copy]. See [LinTree][crate::example::LinTree] for an example of
/// that.
///
/// # no_std note
/// The `callback_*` functions allow operating on the trees without allocations in a
/// no_std-compatible way by calling a provided function on each visited element.
/// This is a bit unwieldy. Additional restrictions are listed in the no_std notes for each function.
///
/// The `iter_*` functions allocate, but return an [Iterator], providing a more comfortable
/// interface.
///
/// # Graph warning
/// If you implement [Treelike] for anything more complex then a DAG you will run into infinite
/// loops with the provided methods. Make sure to avoid loops or override.
///
/// # Traversals and searches
/// Most traversals take a Filter attribute. By passing () you get a pure traversal. By filtering
/// you get a search.
pub trait Treelike: Sized + Copy {
	/// The content of the current node.
	///
	/// If the node does not always contain something
	/// make Content an [Option].
	type Content;

	/// You will have to specify the precise type you use for child iteration.
	/// This also implies that you have to move any closures into free standing functions.
	/// This is an [Iterator] over the children, not the contents of the children.
	type ChildIterator: Iterator<Item = Self>;

	/// Has to return an Iterator over all this nodes direct children.
	///
	/// The exact type sadly has to be specified in [ChildIterator][Treelike::ChildIterator]
	/// as impl Trait is not (yet) usable in Traits.
	fn children(self) -> Self::ChildIterator;

	/// Returns leftmost direct child of this Node. Mostly useful for binary trees.
	fn left(self) -> Option<Self> { self.children().next() }

	/// Returns rightmost direct child of this Node. Mostly useful for binary trees.
	fn right(self) -> Option<Self> { self.children().last() }

	/// Has to produce this nodes [Content][Treelike::Content].
	fn content(self) -> Self::Content;

    /// Finds the content of a leaf node based on a given traversal without backtracking.
    fn leaf_by(mut self, mut f: impl FnMut(Self) -> Option<Self>) -> Self::Content {
        while let Some(next) = f(self) {
            self = next;
        }
        self.content()
    }

	/// Recursively traverses the tree to the very first/leftmost node.
	fn first(self) -> Self::Content {
        self.leaf_by(Self::left)
	}

	/// Recursively traverses the tree to the very last/rightmost node.
	fn last(self) -> Self::Content {
        self.leaf_by(Self::right)
	}

	/// Traverses the tree depth first, post order,
	/// i.e. children's contents are visited before their parents.
	///
	/// The provided callback gets called on each visited node.
	///
	/// You can optionally provide child_filter. It is used to determine which children of a node to visit.
	/// child_filter can be anything that [FilterBuilder] is implemented for.
	///
	/// # Examples
	///
	/// Pass () as filter to just visit all children.
	///
	/// ```
	/// # use treelike::example::LinTree;
	/// # use treelike::Treelike;
	/// # let base = [3, 4, 5, 6, 7];
	/// # let node = LinTree::new(0, &base);
	/// node.callback_dft(
	/// 	|content, depth| {
	/// 		dbg!((content, depth));
	/// 		},
	/// 	(),
	/// 	)
	/// ```
	///
	/// Pass an  `Fn(Self::Content, depth: usize, child: Self) -> bool` to filter.
	/// For example stop at depth 1 and nodes with content 4:
	///
	/// ```
	/// # use treelike::example::LinTree;
	/// # use treelike::Treelike;
	/// # let base = [3usize, 4, 5, 6, 7];
	/// # let node = LinTree::new(0, &base);
	/// node.callback_dft(
	/// 	|content, depth| {
	/// 		dbg!((content, depth));
	/// 		},
	/// 	(|content, depth, child| **content != 4 && depth <= 1)
	/// #   //FIXME: I do not understand why this cast is needed
	///     as for<'r, 's> fn(&'r &usize, usize, &'s LinTree<'_, usize>) -> _,
	/// 	)
	/// ```
	///
	/// # no_std note
	/// A stack is necessary for depth-first traversals. This method uses the call-stack to get
	/// around not using allocations. This should not cause additional runtime costs.
	fn callback_dft<CB: FnMut(Self::Content, usize), F: FilterBuilder<Self>>(
		self,
		callback: CB,
		child_filter: F,
	) {
		callback_dft(self, callback, child_filter, 0);
	}

	/// like [callback_dft][Treelike::callback_dft] but the parents content is visited before
	/// the children's.
	fn callback_dft_pre<CB: FnMut(Self::Content, usize), F: FilterBuilder<Self>>(
		self,
		callback: CB,
		child_filter: F,
	) {
		callback_dft_pre(self, callback, child_filter, 0);
	}

	/// Traverses the tree breadth-first, i.e. one depth-layer at a time.
	/// # Example
	/// ```
	/// # use treelike::example::LinTree;
	/// # use treelike::Treelike;
	/// let base = [3, 4, 5, 6, 7];
	/// let node = LinTree::new(0, &base);
	///
	/// let mut order = Vec::new();
	/// node.callback_bft(|content, depth| order.push(*content));
	///
	/// assert_eq!(&order, &base);
	/// ```
	///
	/// # Performance warning
	/// The default implementation is no_std-compatible, using no allocations. It pays a
	/// substantial performance price for that.
	/// Specifically each node is visited `depth - total_depth` times.
	///
	/// Custom implementations are able and encouraged to override this if possible.
	/// [LinTree][crate::example::LinTree] for example replaces this iterating over its slice.
	///
	/// # no_std Note
	/// A queue is necessary for breadth-first traversals. This method repeatedly traverses to
	/// deeper and deeper depths. This causes additional runtime costs.
	fn callback_bft<CB: FnMut(Self::Content, usize)>(self, callback: CB) {
		self.callback_bft_filtered(callback, ())
	}

	/// Like [callback_bft][Treelike::callback_bft] but allows filtering, thereby disallowing some
	/// optimizations.
	fn callback_bft_filtered<CB: FnMut(Self::Content, usize), F: FilterBuilder<Self>>(
		self,
		mut callback: CB,
		filter: F,
	) {
		let mut depth = 0;
		let mut count = 0;

		loop {
			callback_bft(
				self,
				|content| {
					count += 1;
					callback(content, depth)
				},
				filter,
				depth,
				0,
			);
			if count == 0 {
				break;
			}
			depth += 1;
			count = 0;
		}
	}

	//TODO: how do I build in-order traversals for trees with more then 2 children? maybe first
	//child, content, other children

	#[cfg(feature = "alloc")]
	fn iter_dft<F: FilterBuilder<Self>>(self, filter: F) -> DFT<Self, F> { DFT::new(self, filter) }

	#[cfg(feature = "alloc")]
	fn iter_dft_pre<F: FilterBuilder<Self>>(self, filter: F) -> DFTP<Self, F> {
		DFTP::new(self, filter)
	}

	#[cfg(feature = "alloc")]
	fn iter_bft<F: FilterBuilder<Self>>(
		self,
		filter: F,
	) -> Chain<Once<Self::Content>, BFT<Self, F>> {
		once(self.content()).chain(BFT::new(self, filter))
	}
}
use core::iter::{once, Chain, Once};

fn callback_dft<T: Treelike, CB: FnMut(T::Content, usize), F: FilterBuilder<T>>(
	t: T,
	mut cb: CB,
	f: F,
	depth: usize,
) -> CB {
	let filter = f.build(t.content(), depth, t.children());
	for child in filter {
		cb = callback_dft(child, cb, f, depth + 1)
	}

	cb(t.content(), depth);
	cb
}

fn callback_dft_pre<T: Treelike, CB: FnMut(T::Content, usize), F: FilterBuilder<T>>(
	t: T,
	mut cb: CB,
	f: F,
	depth: usize,
) -> CB {
	cb(t.content(), depth);

	let filter = f.build(t.content(), depth, t.children());
	for child in filter {
		cb = callback_dft_pre(child, cb, f, depth + 1)
	}

	cb
}

fn callback_bft<T: Treelike, CB: FnMut(T::Content), F: FilterBuilder<T>>(
	t: T,
	mut callback: CB,
	f: F,
	limit: usize,
	depth: usize,
) -> CB {
	if depth == limit {
		callback(t.content());
		return callback;
	}

	for child in t.children() {
		callback = callback_bft(child, callback, f, limit, depth + 1)
	}

	callback
}

pub trait FilterBuilder<T: Treelike>: Copy {
	type Filter: Iterator<Item = T>;
	fn build(self, content: T::Content, depth: usize, children: T::ChildIterator) -> Self::Filter;
}

impl<T: Treelike> FilterBuilder<T> for () {
	type Filter = T::ChildIterator;

	fn build(self, _: T::Content, _: usize, children: T::ChildIterator) -> Self::Filter { children }
}

pub struct PseudoCurry<T: Treelike, F: Fn(&T::Content, usize, &T) -> bool, I: Iterator<Item = T>> {
	content: T::Content,
	depth: usize,
	inner_filter: F,
	inner_iter: I,
}

impl<T: Treelike, F: Fn(&T::Content, usize, &T) -> bool, I: Iterator<Item = T>> Iterator
	for PseudoCurry<T, F, I>
{
	type Item = T;
	fn next(&mut self) -> Option<T> {
		// this is basically just filter but with context
		self.inner_iter
			.next()
			.filter(|child| (self.inner_filter)(&self.content, self.depth, &child))
	}
}

//FIXME: this should not contain anonymous lifetimes because that forces casts down the line.
//but if I try to introduce lifetimes I get "unconstrained lifetime parameter (E0207)" even though
//the lifetime is clearly used in F...
impl<T: Treelike, F: Copy + Fn(&T::Content, usize, &T) -> bool> FilterBuilder<T> for F {
	type Filter = PseudoCurry<T, F, T::ChildIterator>;

	fn build(self, content: T::Content, depth: usize, children: T::ChildIterator) -> Self::Filter {
		PseudoCurry {
			content,
			depth,
			inner_filter: self,
			inner_iter: children,
		}
	}
}

#[derive(Clone, Copy)]
pub struct M<T: Clone + Copy>(T);
// any kind of Fn trait, even with incompatible arguments might be implemented on a single type
// you can't have multiple implementations of a trait for multiple Fn-traits.
// so we need to newtype-wrap it..
//FIXME: https://github.com/rust-lang/rust/issues/60074
impl<
		T: Treelike,
		I: Iterator<Item = T>,
		F: Fn(&T::Content, usize, T::ChildIterator) -> I + Copy,
	> FilterBuilder<T> for M<F>
{
	type Filter = I;

	fn build(self, content: T::Content, depth: usize, children: T::ChildIterator) -> Self::Filter {
		(self.0)(&content, depth, children)
	}
}

#[cfg(feature = "alloc")]
pub struct DFT<T: Treelike, F: FilterBuilder<T>> {
	stack: Vec<(T, F::Filter)>,
	filter: F,
}

#[cfg(feature = "alloc")]
impl<T: Treelike, F: FilterBuilder<T>> DFT<T, F> {
	fn new(treelike: T, filter: F) -> Self {
		let stack = Vec::new();
		let mut s = Self { stack, filter };
		s.push(treelike);
		s
	}
	fn push(&mut self, t: T) {
		let filtered = self
			.filter
			.build(t.content(), self.stack.len(), t.children());
		self.stack.push((t, filtered));
	}
}

#[cfg(feature = "alloc")]
impl<T: Treelike, F: FilterBuilder<T>> Iterator for DFT<T, F> {
	type Item = T::Content;
	fn next(&mut self) -> Option<Self::Item> {
		let (node, mut children) = self.stack.pop()?;
		// if we still have children left to visit, visit those first
		if let Some(child) = children.next() {
			self.stack.push((node, children));
			self.push(child);
			self.next()
		} else {
			// else this node is done and we return the content.
			Some(node.content())
		}
	}
}

//FIXME: test these implementations and add methods on Treelike
#[cfg(feature = "alloc")]
pub struct DFTP<T: Treelike, F: FilterBuilder<T>> {
	stack: Vec<F::Filter>,
	filter: F,
	cur: Option<T::Content>,
}

#[cfg(feature = "alloc")]
impl<T: Treelike, F: FilterBuilder<T>> DFTP<T, F> {
	fn new(treelike: T, filter: F) -> Self {
		let stack = Vec::new();
		let mut s = Self {
			stack,
			filter,
			cur: None,
		};
		s.push(treelike);
		s
	}
	fn push(&mut self, t: T) {
		let filtered = self
			.filter
			.build(t.content(), self.stack.len(), t.children());
		self.stack.push(filtered);
		self.cur = Some(t.content());
	}
}

#[cfg(feature = "alloc")]
impl<T: Treelike, F: FilterBuilder<T>> Iterator for DFTP<T, F> {
	type Item = T::Content;
	fn next(&mut self) -> Option<Self::Item> {
		self.cur.take().or_else(|| {
			let mut children = self.stack.pop()?;
			if let Some(child) = children.next() {
				// children is not empty yeet, put it back and push child for next
				// iteration
				self.stack.push(children);
				self.push(child);
			}
			self.next()
		})
	}
}

#[cfg(feature = "alloc")]
// does not return the root nodes content, combine with chain!
pub struct BFT<T: Treelike, F: FilterBuilder<T>> {
	queue: VecDeque<(F::Filter, usize)>,
	filter: F,
}

#[cfg(feature = "alloc")]
impl<T: Treelike, F: FilterBuilder<T>> BFT<T, F> {
	fn new(treelike: T, filter: F) -> Self {
		let queue = VecDeque::new();
		let mut s = Self { queue, filter };
		s.push(treelike, 0);
		s
	}

	fn push(&mut self, t: T, depth: usize) {
		let filtered = self.filter.build(t.content(), depth, t.children());
		self.queue.push_back((filtered, depth));
	}
}

#[cfg(feature = "alloc")]
impl<T: Treelike, F: FilterBuilder<T>> Iterator for BFT<T, F> {
	type Item = T::Content;
	fn next(&mut self) -> Option<Self::Item> {
		let (mut children, depth) = self.queue.pop_front()?;

		if let Some(child) = children.next() {
			self.queue.push_front((children, depth));
			self.push(child, depth + 1);
			Some(child.content())
		} else {
			self.next()
		}
	}
}
