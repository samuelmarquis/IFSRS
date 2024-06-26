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
* The trivial case is of the DISPLAY block, which leaves only insofar as it is presented to the user.
* The non-trivial case is of the ITERATOR block, which binds values from the automation editor to values in the IFS.

We can either directly access the IFS, or simply return a set of requested values and allow the app
to route those values from the Automation Editor to their corresponding values in the IFS.

The question then arises: what the fuck are you talking about?
* Does the app have a list of fields in the IFS that map to parameters?
  * How do you even store a list of the IFS' fields?
* Does the app maintain a copy of the IFS with sentinel values that indicate what goes where?
* Do we let the render thread own the IFS and have it send us a message when it wants a value for a field?
* Do we have to wrap everything in the IFS in some sort of Option type that represents whether it's automated?

or,
* Can we call fn process() (externally for ITERATOR, internally for DISPLAY) with an ID and let it return a value?
* This ID value will be the ID of a specific terminal (NodeId)--the block level is too vague, as iterators should group
all their automation targets into a single block
* 