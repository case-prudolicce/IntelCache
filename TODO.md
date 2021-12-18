# General
## Features
* Links
* ENTRY/DIR/TAG RENAME command
* ENTRY DISPLAY command (Maybe)
* Modular functions
## Future Hardening
* IcCommand validation (Hardening)

# Native IC Client
## Features
* raw data input (for ENTRY CREATE and others)
## Bugs to fix
* new with third arg (dir loc) does not work
* crash on get\_packet (if server restarts)

# Server
## Features
* Server exporting/backups
## Hardening
* General hardening
## Bugs to fix
* DIR CREATE/DELETE doesn't return right response.
* DIR SET <DIR ID> 0 doesn't work.

# Before 1.0
* README.md
## Crates.io publication
* Documentation/Minor tweaks
