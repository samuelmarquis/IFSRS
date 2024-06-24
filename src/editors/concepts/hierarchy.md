# THE NODE STRUCTURE:
NodeGraphEditor has:
* Collection of nodes
* Collection of node-archetypes
* Which node is currently selected
* Where the last click was (for drawing new nodes)

NodeArchetype has:
* name
* list of input names
* list of output names
* default parameters
* category & info about that category (color, ...?)
* node size/shape

Node has:
* name
* collection of input terminals
* collection of output terminals
* collection of parameters
* position
* physical rectangle/drawn object (embeds category info)

Terminal has:
* name
* knowledge about whether it's IN or OUT
* the terminal it connects to

NodeGraphEditor CREATES Nodes FROM NodeArchetypes.
Nodes CREATE Terminals FROM NodeArchetype i/o lists.
NodeGraphEditor DRAWS Nodes, 
Nodes DRAW Terminals, 
Terminals DRAW edges.

When a drag happens:
* if on a Node, open node properties panel & move node
* if on an out Terminal, connect a wire from the terminal
to the cursor while pressed

When a drag is released:
* if on empty space or a node body, do nothing
* if on a terminal, connect to that terminal if possible, overwriting
the existing connection should one exist

^idealist nonsense
# THE PROBLEM:
If Nodes own terminals, then what does a terminal target?
if it targets another terminal, how does it access that?
if it targets a node along with an index, ???

If the NodeGraphEditor owns the terminals, then drawing nodes makes no sense
because the terminals can't tell the nodes where to be.
Can they?

WHAT THE FUCK IS A SIGNAL ?????

## NODES ARE FAKE. TERMINALS ARE THE REAL NODES, BUT THEY'RE EVEN MORE FAKE
## THE PILL: ADJACENCY MATRIX
### 100,000 DEAD REFERENCE COUNTERS