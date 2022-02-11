# The STORAGE module handles the data entries.

## Commands
* ```STORAGE ENTRY CREATE <NEW ENTRY NAME> [UNDER <DIR ID>] <COOKIE>```

	This is for creating new entries with name `<NEW ENTRY NAME>`. The Command body will be the data to use for the new enty.

	`UNDER <DIR ID>` Will create the entry with loc `<DIR ID>`. If it is missing, will default to `0`.
* ```STORAGE ENTRY SHOW [<DIR ID>] <COOKIE>```

	This is to return entry summaries in IntelCache. If `<DIR ID>` is specified, only return summaries with loc `<DIR ID>`
* ```STORAGE ENTRY DELETE <ENTRY ID> <COOKIE>```

	This command deletes entry with id `<ENTRY ID>`.
* ```STORAGE ENTRY SET <ENTRY ID> <DIR ID> <COOKIE>```

	This command will change the loc of an entry with id `<ENTRY ID>` to loc `<DIR ID>`

	It will also change the data of the entry if data is in the body.
* ```STORAGE ENTRY GET <ENTRY ID> <COOKIE>```

	This command will return an entry with id `<ENTRY ID>` with body containing data.
* ```STORAGE DIR CREATE <NEW DIR NAME> [UNDER <DIR ID>] <COOKIE>```

	This is for creating new directories with name `<NEW DIR NAME>`.

	`UNDER <DIR ID>` Will create the entry with loc `<DIR ID>`. If it is missing, the loc will be null (or commonly put, it will have no loc).
* ```STORAGE DIR SHOW [<DIR ID>] <COOKIE>```

	This command will show all directories in the IntelCache if `<DIR ID>` is missing. If it isn’t, it will show all directories in `<DIR ID>`
* ```STORAGE DIR DELETE <DIR ID> <COOKIE>```

	This command will delete a directory with id `<DIR ID>`
* ```STORAGE DIR SET <DIR ID> <NEW DIR LOC ID> <COOKIE>```

	This command will change a directory’s loc (with id `<DIR ID>`) to a directory with id `<NEW DIR LOC ID>`
* ```STORAGE DIR VALIDATE <DIR ID> <COOKIE>```

	This command will return true if `<DIR ID>` is a valid one (false if invalid), with it’s name in the response’s body.
* ```STORAGE SHOW [<DIR ID>] <COOKIE>```

	This Command will return all directories and entries in the IntelCache. If `<DIR ID>` is specified, it will return all on the specific directory id.
* ```STORAGE TAG DIR <DIR ID> <TAG ID> <COOKIE>```

	This command will add a tag to a directory with id `<DIR ID>` with a tag with id `<TAG ID>`
* ```STORAGE TAG UNDIR <DIR ID> <TAG ID> <COOKIE>```

	This command will remove a tag with id `<TAG ID>` from a directory with id `<DIR ID>`
* ```STORAGE TAG ENTRY <ENTRY ID> <TAG ID> <COOKIE>```

	This command will add a tag to an entry with id `<ENTRY ID>` with a tag with id `<TAG ID>`
* ```STORAGE TAG UNENTRY <ENTRY ID> <TAG ID> <COOKIE>```

	This command will remove a tag with id `<TAG ID>` from an entry with id `<ENTRY ID>`
* ```STORAGE TAG CREATE <NEW TAG NAME> <COOKIE>```

	This command will create a tag with name `<NEW TAG NAME>`
* ```STORAGE TAG DELETE <TAG ID> <COOKIE>```

	This command will delete a tag with id `<TAG ID>`
* ```STORAGE TAG SHOW <COOKIE>```

	This command will return all available tags in the response body.
* ```STORAGE EXIT```

	(DEPRECATED) This Command will disconnect the client from the IntelCache node.
