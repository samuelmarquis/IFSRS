# THE NODE STRUCTURE: (OUTDATED)...
NodeGraphEditor has:
* Collection of nodes
* Collection of node-archetypes
* Collection of terminals
* Which node is currently selected

NodeArchetype has:
* name
* list of input names
* list of output names
* default parameters
* category & info about that category (color, ...?)
* node size/shape

Node has:
* name
* \# of input terminals
* \# of output terminals
* collection of parameters?
* position
* physical rectangle/drawn object

Terminal has:
* name
* knowledge about whether it's IN or OUT

NodeGraphEditor DOES:
* CREATES Nodes FROM NodeArchetypes.
* CREATES Terminals FROM NodeArchetype i/o lists.
* MAINTAINS an internal graph of terminal edges, ensures all connections are from OUT to IN
* DRAWS Nodes, Terminals, & Edges

When a drag happens:
* if on a Node, set selected node & move node (grip snap)
* if on an OUT Terminal, connect a wire from the terminal
to the cursor while pressed

When a drag is released:
* if on empty space or a node body, do nothing
* if on an IN terminal, connect to that terminal if possible, overwriting
the existing connection should one exist

## THE PROBLEM:
Terminals don't know where they are. Nodes tell the editor where their terminals should be,
and then the editor has to draw the terminals from the positions they're given in.
However... the editor doesn't maintain an explicit association between the terminals it contains, 
and the nodes that they "belong" to. If we give terminals back to the nodes,
it leads to the whole reference-counting/borrowing nightmare.
We could have "fake" terminals that we draw, owned by nodes, and "real" terminals that are entries in the graph,
but then we get into shared state hell. 

Maybe we create some kind of parent-struct that contains both nodes and terminals, but this hardly seems
more ideal than just having the nodes own the terminals--same level of indirection with likely just as many
borrowing headaches.

Whatever solution has to also consider that nodes may be deleted, and their terminals + any associated edges
will have to be deleted with them.

Furthermore,

### WHAT THE FUCK IS A SIGNAL ?????