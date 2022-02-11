IntelCache, an extensible cloud storage solution.
=

Intelcache is a combo of a client + server that aims at providing the general populace with cloud technology that prioritize transparency, extensibility and privacy. It comes with a server and a native client that will connect to the local server then communicate using a predefined set of commands. Read more on the server and client by reading their respective README Pages.

Motivation
-

IntelCache main motivation was mostly as an alternative to proprietary cloud storage technologies that are closed sources and somewhat vague when it comes to their privacy standard. It also was originally made as a note taking app that evolved past simply storing text.

Modules
-

Modules are the core aspect of IntelCache. They consist of a set of defined packets that consists of a header and body, containing the command/response and data respectively. The CORE and STORAGE modules are loaded by default with every instance of IntelCache (as of 2.2.2). Read more on the module's respective README.
