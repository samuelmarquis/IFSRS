# THE AUTOMATION EDITOR
AutomationEditor has:
* Collection of blocks
* Collection of block-archetypes
* Graph of terminals & edges
* Other state relevant to drawing

BlockArchetype has:
* name
* category & color
* BlockType(enum containing an enum)
* list of input names
* list of output names
* size

Block has:
* name
* BlockType
* vec of indexes of input terminals in the graph
* vec of indexes of output terminals in the graph
* position
* state for drawing

Terminal has:
* name
* io enum{IN, OUT}
* value
* position, unfortunately

AutomationEditor DOES:
* CREATES Blocks FROM BlockArchetypes.
* CREATES Terminals FROM BlockArchetype i/o lists.
* MAINTAINS an internal graph of terminal edges by ensuring: 
  * Connections can only be created from OUT to IN 
  * When a connection is created remove all other connections to that IN first (should they exist)
* DRAWS Blocks, Terminals, & Edges

When a drag happens:
* if on a block, set selected block & move block (todo: grid snap)
* if on an OUT terminal, connect a wire from the terminal
to the cursor while pressed
* todo: if on an IN terminal with a connection, grab that connection to reroute to a different IN from the same OUT

When a drag is released:
* if on empty space or a block body, do nothing
* if on an IN terminal, connect to that terminal if possible, overwriting the existing connection should one exist

## SIGNALS
Each TARGET block has a value that leaves the automation editor. 
We use the `process(&NodeIndex)` function to get an `Option<f32>` value.
* The trivial case is of the DISPLAY block, which leaves only insofar as it is presented to the user.
  * The automation editor collects a list of DISPLAYs that don't have a value, and calls `process` on them
    every frame until they're able to get one.
* The non-trivial case is of the ITERATOR block. 
  * Some part of the program will call into `process` with the NodeIndex returned from `update_target`
    and get the value for that specific parameter.

The `process` function is well-documented.

## THE PROBLEM:
Detect changes in the graph (timestamp, the graph structure, values of constants) to force a recompute.
Recompute forcing has to last more than a full cycle, or the process queue needs to be filled after all possible
changes to the graph are made, meaning we either have to iterate over blocks twice, or do a lot more stuff with gross state.