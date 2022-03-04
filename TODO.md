# To Implement
* systemd module
* AUR package
* E2E encryption
* Decentralization
* Testing (Docker)
* Client server binary merge

## Modules
* CALENDAR 
* NOTE
* PROFILE
* HABIT

### CORE module
* CORE CAPABILITIES

### STORAGE module
* STORAGE Links
* STORAGE ENTRY labels
* STORAGE ENTRY GET \<DIR\>
* STORAGE ENTRY MAKE \<ARCHIVE\>
* STORAGE ENTRY SHOW with PUBLIC and PRIVATE

## Native IC Client
* ls a prints as tree
* Password prompt for login

## Server
* --export,--import and --teardown handle ipfs
* --raw\_dump (Dumping all files from user in archive)

# (WOF) Known Bugs to fix

## Server
* TBD

### STORAGE MODULE
* STORAGE ENTRY SHOW will return Error is there isn't any entries in the location
* STORAGE TAG UNDIR returns OK! even if it fails.
* [UV] STORAGE ENTRY SET \<NEW NAME\> doesn't unwrap the surrounding parantheses.
* [UV] STORAGE DIR SET \<NEW NAME\> doesn't unwrap the surrounding parantheses.
* [TV] STORAGE ENTRY SHOW 0 \<COOKIE\> returns error.

## Native IC Client
* `ls d` is broken (doesn't include the PUBLIC PRIVATE keyword)
* `ls f` is broken (sends 0 regardless of current pwd)

# (WOF) MISC/MARKERS
* (B2.0,Last) Cargo tomls
* (WO) (B2.0,Before Last) Cargo Docs.

## Native IC Client
* TBD

## LIB
* lib\_backend @ get\_entry: Write file directly from ipfs instead of holding in memory (TODO:1)
