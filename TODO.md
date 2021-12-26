# General
## Features
* Links
* Entry labels
* calendar
* ENTRY/DIR/TAG RENAME command
* Modular future function
* Users and data visibility (public/private)
* e2e encryption
* Decentralization
## Future Hardening
* IcCommand validation (Hardening)

# Native IC Client
## Features
* README
* ls a prints as tree
## Bugs to fix
* new with third arg (dir loc) does not work
* crash on get\_packet (if server restarts)
* mkdir <name> <dir id> doesn't work
* ls is pretty bugged in general
* will crash (no EDITOR var) on edit

# Server
## Features
* Commands (start,stop (pid signals), export)
* logging 
## Bugs to fix
* DIR CREATE/DELETE doesn't return right response.
* DIR SET <DIR ID> 0 doesn't work.
* Server crashes if client disconnects (Handle ungraceful exits from client)
