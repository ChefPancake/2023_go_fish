# TODO K:
- [x] Animate fish
  - [x] movement in short bursts
  - [x] staying in certain zone
  - [x] stretch and squash based on movement
  - [x] randomized burst lengths
  - [x] spawn fish in lanes w slight variation
- [ ] catch fish
  - [x] create hook, player moves it with wasd
  - [x] on spacebar, remove one fish under hook
    - [x] adjust catch point to be at mouth
    - [x] when in range, freeze the fish and the hook
    - [x] when space is pressed, remove the fish and unfreeze the hook
  - [x] instead of removing, arc it up to catch area
  - [x] pull fish up to water surface when catching, then arc in
  - [x] On critical catch, arc fish immediately into stack
  - [x] sit it on a stack
  - [ ] animate it flopping when caught
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
- [ ] add snail
  - circle for now
  - [ ] moves across bottom between rocks
  - [ ] game ends when he travels across
- [x] squidge out smaller caught fish if a larger is caught
  - [x] remove from stack if a larger fish is added
  - [x] arc them out of the stack
  - [x] return them to their original pos.y
  - [x] tone down the vel_x
- [ ] after squidging, drop top fish to new position
- [ ] add x drag to water during return
- [ ] add fish chasing
  - when near and facing the hook, fish will chase it. 
  - fish will stay within lane, leaving the lane will cause the fish to give up and return to original pos.y
- [ ] add casting
  - [ ] hook starts out of water
  - [ ] press space to cast
    - goes to center of water
    - maybe add holding space to cast nearer or farther? might be unnecessary
- [ ] line updates
  - [ ] add offsets for each bear frame
  - [ ] tighten line when reeling
  - [ ] when reeling, move fish.x slightly towards bear.x (towards centroid)
- [ ] animate bear
  - [ ] before casting, switch to cast frame
  - [ ] when hook in air, switch to fishing frame
  - [ ] when reeling, switch to reel frame
  - [ ] critical catch
    - [ ] hitstop
      - freeze all fish: swimming, reeling, and flying
    - [ ] alternate two critical frames N times
  - [ ] when caught fish goes flying, switch to catch frame
- [x] reset level when last fish lands on stack and nothing is knocked out
- [ ] reorganize
- [ ] refactor interpolation data/funcs

# TODO E:
- Fix bear spot perspective
- add stump
- flatten/lower water
- move stack spot away from edge
- name game
- need bear frames
  - idle? maybe
  - cast throw

# Reorganization plan

I've been trying to figure out exactly where I want to draw the boundaries for each "module" once I split this main file apart. 
I think what mostly needs to happen is that larger state changes need to be pulled out into events. This will be super relevant 
once we get to audio as I don't want to have to pepper in audio on every update system, where I really just need it to emit a 
sound when certain things happen.

So I think the plan is - look at every update system that takes in `Commands`, and have it emit an event instead. Then have event 
listeners/readers that actually submit the commands and/or otherwise react to the events each be their own systems. Once those 
systems, we'll see what sort of shape emerges out of it. It would be nice if fish as a concept could be pulled out into their own
plugin, hook into its own, the bear into its own, etc, and they just pass events around to change state. This could introduce its
own challenges, as it does in any distributed system.