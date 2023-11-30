# TODO K:
- [x] Animate fish
  - [x] movement in short bursts
  - [x] staying in certain zone
  - [x] stretch and squash based on movement
  - [x] randomized burst lengths
  - [x] spawn fish in lanes w slight variation
- [x] catch fish
  - [x] create hook, player moves it with wasd
  - [x] on spacebar, remove one fish under hook
    - [x] adjust catch point to be at mouth
    - [x] when in range, freeze the fish and the hook
    - [x] when space is pressed, remove the fish and unfreeze the hook
  - [x] instead of removing, arc it up to catch area
  - [x] pull fish up to water surface when catching, then arc in
  - [x] On critical catch, arc fish immediately into stack
  - [x] sit it on a stack
- [x] Spawn multiple sizes of fish
  - [x] Larger fish in the background
  - [x] randomize placement
- [x] draw fishing line
- [x] Incorporate images
  - [x] Background
    - move stuff into position
  - [x] Bear - 1 frame
    - move line to the rod
  - [x] Fish
    - Spawn one of each
  - [x] Stacked Fish
    - change sprite to the top profile
- [x] add snail
  - [x] moves across bottom between rocks
  - [x] game ends when he travels across
    - snail sends event when he reaches the end
- [x] squidge out smaller caught fish if a larger is caught
  - [x] remove from stack if a larger fish is added
  - [x] arc them out of the stack
  - [x] return them to their original pos.y
  - [x] tone down the vel_x
- [x] add casting
  - [x] hook starts out of water
  - [x] press space to cast
    - goes to center of water
    - maybe add holding space to cast nearer or farther? might be unnecessary
- [x] line updates
  - [x] add offsets for each bear frame
  - [x] tighten line when reeling
  - [~] when reeling, move fish.x slightly towards bear.x (towards centroid)
    - decided to not do this, don't want the fish to clip into the ground and don't want to reorient the fish just yet.
- [x] animate bear
  - [x] before casting, switch to cast frame
  - [x] when hook in air, switch to fishing frame
  - [x] when reeling, switch to reel frame
  - [x] critical catch
    - [x] alternate two critical frames N times
  - [x] when caught fish goes flying, switch to catch frame
  - [x] squish and stretch
    - [x] on cast, stretch forward then relax
    - [x] when reeling, pulse/shake fast
    - [x] on catch, stretch up then relax
    - [x] when fishing, slow pulse
    - [x] on crit, ???
    - [x] squish from feet
- [x] reset level when last fish lands on stack and nothing is knocked out
- [x] reorganize
- [x] audio
  - [x] music
    - [jummbox](https://jummbus.bitbucket.io)
  - [x] sound effects
- [ ] animate fish flopping when caught
- [ ] refactor interpolation data/funcs
- [ ] hitstop
  - freeze all fish: swimming, reeling, and flying
- [ ] after squidging, drop top fish to new position
- [ ] add x drag to water during return
- [ ] add fish chasing
  - when near and facing the hook, fish will chase it. 
  - fish will stay within lane, leaving the lane will cause the fish to give up and return to original pos.y


# Reorganization Notes
Currently there is a heirarchy of plugins. From top to bottom, where the top knows about everything and the bottom doesn't know anything:

- catch_stack
- hook
- fish
- physics
- core

Some of these relationships might be unavoidable, like the hook knowing about fish. But a lot of these relationships are maintained for their events. This could be flattened out if events were owned by core, and then everything referenced that instead. Similar to a modular monolith, where the interfaces are at the center and all the modules reference those to either depend on them or implement them. I'll leave it alone for now as I
don't want this to get in the way of creating features and I don't want to
guess wrong on what the structure should be.

