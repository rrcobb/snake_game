# snake

from https://snake.rustbridge.com/

## Notes

- intended for a live session, not a self-led async kind of thing
- lots of copy/paste coding, maybe that's okay
- the end result code is kinda bad!
- we don't make the whole snake game!

- all mutable all the time, BABY
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
  - or, there might be a factoring of the code that feels more appropriate

- nice to introduce, not introduced:
  - nicer iteration patterns (use the vec!)
  - enum for the direction
  - using clone
  - dbg!, Clone, Debug

- https://github.com/rustbridge/rusty-snake-book/issues

## Making the game work

### DONE
- clear path behind snake (clear whole grid)
- snake is a vec, instead of representing as a path on the grid
- pause (spacebar)
- how should the game end?
  - for now: console.log a loss, end
- snake die on self-collision
- show dot
- eat dot: snake length + 1, new dot
- don't place the dot on the snake
- show text on the screen

### TODO

- text
    - start screen
      - So, before the event loop starts?
    - end screen
     - So, after the event loop ends?
    - scores
    - frames
- show a menu, handle menu navigation keyboard events
- respond to click events on a menu
- wrap at the edges (currently, die at edges)
- snake color

- some refactoring
    - grid to store cols and rows?
    - state object for the game?
    - somewhere to keep arbitrary consts (Settings struct?)
    - associated methods instead of free ones?
    - rename the _init functions to something else

possible:
- speed increase when eat a dot?
- game settings: size, speed, colors, icons?
- menu? pause menu?
- use a worker thread to speed something up?
- show timing information
- buffer the input so that two keypresses in a row don't auto-die
- make 'backwards' disallowed instead of end of game
- smoother animations
- profiling? care more abt speed and memory use?
  - cache the rng for the dot random location
  - where is the actual speed cost?
- use some kind of fancy data structure somehow (for what?)
- tests
- handle window resize well
