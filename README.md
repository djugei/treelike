# Treelike

<!-- cargo-sync-readme start -->

This crate tries to provide a common trait for all kinds of trees. Two reasons for that:

## Interoperability
Using a common trait allows third parties to switch out tree implementations seamlessly

## Automatisation
If you are implementing a tree this `Treelike` only requires you to implement two methods on
your nodes, `content` to return its contents and `children` to list its children.

Many kinds of traversals and searches are then provided for free. I found myself implementing
the same methods over and over on different trees, so that is my main motivation.


# no_std
This crate tries to stay no_std compatible, but provides more functionality if allocations are
available. The relevant types and methods contain a no_std section to discuss functionality and
limitations.

<!-- cargo-sync-readme end -->

# Contributing
Please symlink the hooks to your local .git/hooks/ directory to run some automatic checks before commiting.

    ln -s ../../hooks/pre-commit .git/hooks/

Please install rustfmt and cargo-sync-readme so these checks can be run.

    rustup component add rustfmt
    cargo install cargo-sync-readme

Please execute `cargo-sync-readme` when you change the top-level-documentation.
Please run `cargo fmt` whenever you change code. If possible configure your editor to do so for you.