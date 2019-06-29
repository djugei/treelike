#![cfg_attr(not(feature = "std"), no_std)]

pub mod example;

/// The main Trait of the crate.
/// Provides many kinds of iterations and searches on trees.
///
/// Just like [Iterator] most functions are provided,
/// only [children][Treelike::children] and [content][Treelike::content] need to be implemented.
///
/// Should probably be implemented on references of the node-type,
/// unless your node itself is already Copy.
/// In case you implement Treelike directly on your Type, you might need to implement [Clone] and
/// [Copy] manually to avoid placing [Clone] and [Copy]-bounds on its type parameters, which is
/// what derive does.
///
/// # no_std note
/// The `callback_*` functions allow operating on the trees without allocations in a
/// no_std-compatible way by calling a provided callback function on each visited element.
/// This is a bit unwieldy. Additional restrictions those are listed in the no_std notes for each function.
///
/// The `iter_*` functions allocate, but return an [Iterator], providing a more comfortable
/// interface.
///
/// # Graph warning
/// If you implement [Treelike] for anything more copmlex then a DAG you will run into infinite
/// loops with the provided methods. Make sure to avoid loops or override.
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

	/// Returns leftmost direct child of this Node. Mostly usefull for binary trees.
	fn left(self) -> Option<Self> { self.children().next() }

	/// Returns rightmost direct child of this Node. Mostly usefull for binary trees.
	fn right(self) -> Option<Self> { self.children().last() }

	/// Has to produce this nodes [Content][Treelike::Content].
	fn content(self) -> Self::Content;

	/// Recursively traverses the tree to the very first/leftmost node.
	fn first(self) -> Self::Content {
		if let Some(next) = self.left() {
			next.first()
		} else {
			self.content()
		}
	}

	/// Recursively traverses the tree to the very last/rightmost node.
	fn last(self) -> Self::Content {
		if let Some(next) = self.right() {
			next.last()
		} else {
			self.content()
		}
	}

	/// Traverses the tree depth first, post order,
	/// i.e. chilrdens contents are visited before their parents.
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
	/// # use tree_trait::example::LinTree;
	/// # use tree_trait::Treelike;
	/// # let base = [3, 4, 5, 6, 7];
	/// # let node = LinTree::new(0, &base);
	/// node.callback_dft(|content, depth| {dbg!((content, depth));}, ())
	/// ```
	///
	/// Pass an  `Fn(Self::Content, depth: usize, child: Self) -> bool` to filter.
	/// For example stop at depth 1 and nodes with content 4:
	///
	/// ```
	/// # use tree_trait::example::LinTree;
	/// # use tree_trait::Treelike;
	/// # let base = [3usize, 4, 5, 6, 7];
	/// # let node = LinTree::new(0, &base);
	/// node.callback_dft(
	///     |content, depth| {dbg!((content, depth));},
	///     (|content, depth, child| **content != 4 && depth <= 1)
	/// #   //FIXME: i do not understand why this cast is needed
	///     as for<'r, 's> fn(&'r &usize, usize, &'s LinTree<'_, usize>) -> _,
	///     )
	/// ```
	///
	/// # no_std note
	/// A stack is nescesary for depth-first traversals. This method uses the call-stack to get
	/// around not using allocations. This should not cause additional runtime costs.
	fn callback_dft<CB: FnMut(Self::Content, usize), F: FilterBuilder<Self>>(
		self,
		callback: CB,
		child_filter: F,
	) {
		callback_dft(self, callback, child_filter, 0);
	}

	/// like [callback_dft][Treelike::callback_dft] but the parents content is visited before
	/// the childrens
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
	/// # use tree_trait::example::LinTree;
	/// # use tree_trait::Treelike;
	/// let base = [3, 4, 5, 6, 7];
	/// let node = LinTree::new(0, &base);
	///
	/// let mut order = Vec::new();
	/// node.callback_bft(|content, depth| order.push(*content));
	///
	/// assert_eq!(&order, &base);
	/// ```
	///
	/// # Performane warning
	/// The default implementation is no_std-compatible, using no allocations. It pays a
	/// substantial performace price for that.
	/// Specifically each node is visited `depth - total_depth` times.
	///
	/// Custom implentations are able and encouraged to override this if possible.
	/// LinTree for example could replace this with simply iterating over its slice.
	///
	/// # no_std Note
	/// A queue is nescesary for breadth-first traversals. This method repeatedly traverses to
	/// deeper and deeper depths. This causes additional runitme costs.
	fn callback_bft<CB: FnMut(Self::Content, usize)>(self, mut callback: CB) {
		let mut depth = 0;
		let mut count = 0;

		loop {
			callback_bft(
				self,
				|content| {
					count += 1;
					callback(content, depth)
				},
				depth,
			);
			if count == 0 {
				break;
			}
			depth += 1;
			count = 0;
		}
	}

	//TODO: dfs
	//TODO: how do i build in-order traversals for trees with more then 2 children? maybe first
	//child, content, other children
}

fn callback_dft<T: Treelike, CB: FnMut(T::Content, usize), F: FilterBuilder<T>>(
	t: T,
	mut cb: CB,
	f: F,
	depth: usize,
) -> CB {
	let filter = f.build(t.content(), depth);

	for child in t.children().filter(|t| filter.filter(t)) {
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

	let filter = f.build(t.content(), depth);
	for child in t.children().filter(|t| filter.filter(t)) {
		cb = callback_dft_pre(child, cb, f, depth + 1)
	}

	cb
}

fn callback_bft<T: Treelike, CB: FnMut(T::Content)>(t: T, mut callback: CB, depth: usize) -> CB {
	if depth == 0 {
		callback(t.content());
		return callback;
	}

	for child in t.children() {
		callback = callback_bft(child, callback, depth - 1)
	}

	callback
}

pub trait FilterBuilder<T: Treelike>: Copy {
	type Filter: Filter<T>;

	fn build(self, content: T::Content, depth: usize) -> Self::Filter;
}

pub trait Filter<T: Treelike> {
	fn filter(&self, child: &T) -> bool;
}

impl<T: Treelike> FilterBuilder<T> for () {
	type Filter = ();

	fn build(self, _: T::Content, _: usize) -> () { () }
}

impl<T: Treelike> Filter<T> for () {
	fn filter(&self, _child: &T) -> bool { true }
}

pub struct PseudoCurry<T: Treelike, F: Fn(&T::Content, usize, &T) -> bool> {
	content: T::Content,
	depth: usize,
	inner_filter: F,
}

//FIXME: this should not contain anonymous lifetimes because that forces casts down the line.
//but if i try to introduce lifetimes i get "unconstrained lifetime parameter (E0207)" even though
//the lifetime is clearly used in F...
impl<T: Treelike, F: Copy + Fn(&T::Content, usize, &T) -> bool> FilterBuilder<T> for F {
	type Filter = PseudoCurry<T, F>;

	fn build(self, content: T::Content, depth: usize) -> Self::Filter {
		PseudoCurry {
			content,
			depth,
			inner_filter: self,
		}
	}
}

impl<T: Treelike, F: Fn(&T::Content, usize, &T) -> bool> Filter<T> for PseudoCurry<T, F> {
	fn filter(&self, child: &T) -> bool { (self.inner_filter)(&self.content, self.depth, child) }
}