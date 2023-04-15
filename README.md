This is a small programming language, for the purpose of prototyping how [Typst](https://github.com/typst/typst) would look like with its impurities unified as algebraic effects.

It is not particularly efficient, but only for the sake of simplicity -- there is nothing inherent that blocks it from being optimized (at least compared to how an untyped lambda calculus [/ an untyped PL] could be optimized).

Note that:

- This does not implement full algebraic effects; in particular, the current implementation only allows for resuming the computation once. Full algebraic effects would allow an effect to resume computation any number of times (including zero).

  The reason for this is nothing fundamental -- simply that it would likely be much simpler to optimize performance when there is a single resumption, and that would likely suffice for Typst.
- This does not let the user of the programming language to define their own effects and handlers.

  Our purpose with algebraic effects in Typst is much more to streamline how the compiler does things internally, as well as refactor some bits that are currently "callback based" (e.g. `location`, `state::display`, `styles`) into something that much more versatile and imperative looking (and thus more intuitive).
