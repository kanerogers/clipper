# clipper
A game about paperclips.

Here's the [pitch](https://github.com/kanerogers/clipper/issues/1) and here's the [prototype](https://github.com/kanerogers/clipper/issues/2).

## Running the game
I'm experimenting with ways to iterate on a game as fast as possible. At the moment I'm using [hot-lib-reloader](https://docs.rs/hot-lib-reloader/latest/hot_lib_reloader/) to link the "game logic" (stored in the helpfully named "game" crate) as a linked library at runtime, updating the library when it changes. This setup is not without its downsides, and requires a little bit of fuckery to get working correctly (see the `common` crate, which essentially stores any dependencies shared between the `game` crate and anything else to avoid Dreaded Dependency Hell). But overall, it's resulted in a very pleasant development experience.

To witness this magic, open two terminals, one running:

`cargo watch -w game -w common -x 'build -p game'`

and the other running:

`cargo run --target-dir target-bin`

And all your wildest dreams will come true.
