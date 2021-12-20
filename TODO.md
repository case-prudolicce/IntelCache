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
* raw data input (for ENTRY CREATE and potential others)
* ls a prints as tree
## Bugs to fix
* new with third arg (dir loc) does not work
* crash on get\_packet (if server restarts)
* mkdir <name> <dir id> doesn't work
* ls on ROOT dir acts like ls a (when it shouldn't).
* ls f/d acts like ls f/da (when it should do ls f/d on pwd)
* will crash (no EDITOR var) on edit (TBIF)

# Server
## Features
* Commands (start,stop (pid signals), export)
* logging 
## Bugs to fix
* DIR CREATE/DELETE doesn't return right response.
* DIR SET <DIR ID> 0 doesn't work.
* Server crashes if client disconnects (TBIF)
