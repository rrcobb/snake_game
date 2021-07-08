# snake

from https://snake.rustbridge.com/

## Notes

- intended for a live session, not a self-led async kind of thing
- lots of copy/paste coding, maybe that's okay
- the end result code is kinda bad
- we don't make the whole snake game!

- all mutable all the time
- a little weird about both taking mutable references _and_ returning an updated
    value
- vecs know their length! no need to pass around extra lengths for the grid size
- could, but don't need to, do some impls on objs, for a more objecty feel
- a little weird abt error handling
      - maybe should panic instead of printing? also, not needed
- doesn't actually use the thread for work, I don't think?
- the modularization is a little weird
  - types.rs is unconventional?
  - it also might not be needed? like, we can do all this in a big main function


- nice to introduce, not introduced:
  - nicer iteration patterns (use the vec!)
  - enum for the direction
  - using clone
  - dbg!, Clone, Debug

- https://github.com/rustbridge/rusty-snake-book/issues

