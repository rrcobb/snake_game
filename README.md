# snake

from https://snake.rustbridge.com/

## Notes

### On the original tutorial

- intended for a live session, not a self-led async kind of thing
- lots of copy/paste coding, maybe that's okay
- the end result code is kinda bad!
- we don't make the whole snake game!

### on the code produced as the tutorial endpoint

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

- nice to introduce so that people can build cool things:
   - sdl2 ttf / font
   - controlling the speed / game loop

- https://github.com/rustbridge/rusty-snake-book/issues


### my own code

- Sticking everything into a big object is a little annoying, mostly bc handling
    the inputs means taking a mutable borrow to the Game object, https://doc.rust-lang.org/nomicon/borrow-splitting.html
    - fixes: 
      - make game state and 'internals' separate
      - push all the events into a buffer 
    - split object per that page, mutate with e.g. `*direction = x;`

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

- refactoring
    - state object for the game
    - somewhere to keep shared initialized stuff (rng, font, canvas, eventpump,
        texturecreator, etc)
    - somewhere to keep arbitrary consts - Settings struct
    - add a game state enum
      - instead of needing to panic to break
    - game loop: 
    - Options for game loop: 
        1 Sleep for 1/framerate, minus time it took to update the state / render
          - https://gameprogrammingpatterns.com/game-loop.html#take-a-little-nap
        2 Update by fluid amount of game seconds
          - https://gameprogrammingpatterns.com/game-loop.html#one-small-step,-one-giant-step
        3 update with fixed time step, render variable time step
          - https://gameprogrammingpatterns.com/game-loop.html#play-catch-up
        - Probably want 1, since it's simple and we don't have to mess with it
        - Need to keep the snake on the grid
        - need to animate the snake between the cells
        - need to only update the snake's actual _cell_ once per 5 renders
- smoother animations
- smooth animation
    - remove explicit grid, do drawing without the grid
    - "implicit grid!"
    - refactor to separate concepts of snake speed and frames
- fix snake jitter (animate in the direction of the next cell!)
- pressing backwards, don't auto-die
  - make 'backwards' disallowed instead of end of game
- score
  - ongoing
- speed increase when eat a dot
- drop unneeded Cell type
- start screen
  - before the event loop starts, probably its own game status
  - status: Start, shows text in the center
  - some button press to start the game
  - necessary refactors: 
    - move frame into game
    - make game loop simpler
    - render differently based on game status
- end screen
  - after the event loop ends, the Over game status
  - restart with message
- centered messages on screens

### TODO

#### Features

- persisted high scores
- playable area separated from full window area
- pause menu
- handle menu navigation keyboard events
- hjkl for navigation
- respond to click events on a menu
- settings menu
- wrap at the edges (currently, die at edges) (as a setting?)
- (user editable) game settings: size, speed, colors, icons?
- snake color
- snake sprite (head, tail)
    - other sprites (dot, what else?)
- rainbow mode (change bg color, what other colors)
- (buffer the input) 
- pop / burst of bg color when eating dot
- windows build, so keely can download
- handle window resize well

code
- show timing information
- profiling? care more abt speed and memory use?
  - cache the rng for the dot random location
  - where is the actual speed cost?
- use a worker thread to speed something up?
- use some kind of fancy data structure somehow (for what?)
- tests
- No, for now: refactor: render_cell from draw_snake and draw_dot
