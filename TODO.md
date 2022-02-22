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

## Native IC Client
* ls a prints as tree

## Server
* --export,--import and --teardown handle ipfs
* --raw\_dump (Dumping all files from user in archive)
* logging 

# Known Bugs to fix

## Server
* (B2.0) crash when peer resets it's connection.
* (B2.0) Logging out crashes (CORE ACCOUNT LOGOUT)

### STORAGE MODULE
* (B2.0) crash on some commands
	* STORAGE ENTRY SHOW (On invalid input)
* STORAGE TAG UNDIR returns OK! even if it fails.
* (B2.0) STORAGE SHOW ignores visibility.
* (B2.0) STORAGE ENTRY SET \<NEW NAME\> doesn't unwrap the surrounding parantheses.
* (B2.0) STORAGE DIR SET \<NEW NAME\> doesn't unwrap the surrounding parantheses.

## Native IC Client
* (B2.0) `ls f` isn't using a DIR ID at all.
* (B2.0) `ls d` isn't using a DIR ID at all.
* (B2.0) `new` without arguments does not remove the newline.
* (B2.0) `new` with arguments sets pwd to one when pwd is 0.
* (B2.0) `fetchusers` without arguments crashes the client.
* (B2.0) `set` without arguments crashes the client.
* (B2.0) `edit` reset the entry's loc to 0

# MISC/MARKERS
* (B2.0) READMEs Readthrough.

## LIB
* lib\_backed @ get\_entry: Write file directly from ipfs instead of holding in memory (TODO:1)
