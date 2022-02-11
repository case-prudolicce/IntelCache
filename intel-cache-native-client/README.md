# The Native IntelCache Client serves to interact with a local IntelCache Server

It works by translating input commands to corresponding module commands.

## Input commands
* ```fetchusers <USERNAME>```
	
	This command will translate to `CORE FETCH USER <USERNAME>` return all global ids for a given username.
* ```login <GLOBAL ID> <PLAINTEXT PASSWORD>```
	
	This command will translate to `CORE LOGIN <GLOBAL ID> <SHA-512 HASHED PASSWORD>` and set the cookie if valid.
* ```ls [f|a|d]```
	
	This command will list files, directory or both.
	
	* `ls` without arguments lists all files and directories under the current directory
		
		It translates to `STORAGE SHOW <CURRENT DIRECTORY ID> <COOKIE>`
		
	* `ls f` will display files under the current directory
		
		It translates to `STORAGE ENTRY SHOW <CURRENT DIRECTORY ID> <COOKIE>`
		
	* `ls d` will do the same for directories
		
		It translates to `STORAGE DIR SHOW <CURRENT DIRECTORY ID> <COOKIE>`
		
	* `ls a` will display all files and all directories for the given user.
		
		It translates to `STORAGE SHOW <COOKIE>`
* ```cd <DIR ID>```
	
	This command will change the current directory to `<DIR ID> <COOKIE>` if valid.
	
	It doesn't translates directly to anything, however it makes a `STORAGE DIR VALIDATE <DIR ID> <COOKIE>` call.

* ```mkdir <DIR NAME>```
	
	This command will create a new directory under the current one.
	
	This command will translate to `TBD, BROKEN`.

* `new/import`
	* `new` creates a new entry as a text file, opening it in vim.
		
		This command will translate to `TBD, BROKEN`.
	* `import [file path] [entry name]` imports an already created file.
		
		This command will translate to `STORAGE ENTRY CREATE <ENTRY NAME> <COOKIE>`.
* `get <entry id>`
	
	This command will download an entry at the current path.
	
	This command will translate to `STORAGE ENTRY GET <ENTRY ID> <COOKIE>`.
* `rm <entry id>`
	
	rm will delete an entry with id `<entry id>`.
	
	This command will translate to `STORAGE ENTRY DELETE <ENTRY ID> <COOKIE>`.
* `mv <ID>[/] <DIR ID>`
	
	This command will change the loc of a directory or file.
	
	If `<ID>` has an appended "/", that id is a directory id.
* `rmdir <DIR ID>`
	
	This command will remove a directory.
	
	This command will translate to `STORAGE DIR DELETE <DIR ID> <COOKIE>`.
* `mktag <TAG NAME>`
	
	Will create a tag
	
	This command will translate to `STORAGE TAG CREATE <TAG NAME> <COOKIE>`.
* `tag <ID>[/] <TAG ID>`
	
	Will tag either a directory or entry with the tag id.
	
	When specifying a directory as a target, add '/' to the end.
	
	This command will translate to `STORAGE TAG DIR <DIR ID> <TAG ID> <COOKIE>`. or
	
	 `STORAGE TAG ENTRY <ENTRY ID> <TAG ID> <COOKIE>` depending if `<ID>` has "/" at the end.
* `untag <ID>[/] <TAG ID>`
	
	Will untag either a directory or entry with the tag id.
	
	When specifying a directory as a target, add '/' to the end.
	
	This command will translate to `STORAGE TAG UNDIR <DIR ID> <TAG ID> <COOKIE>`. or
	
	 `STORAGE TAG UNENTRY <ENTRY ID> <TAG ID> <COOKIE>` depending if `<ID>` has "/" at the end.
* `showtags`
	
	Will show tags
	
	This command will translate to `STORAGE TAG SHOW`.
* `rmtag <TAG ID>`
	
	Will remove a tag
	
	This command will translate to `STORAGE TAG DELETE <TAG ID> <COOKIE>`.
* `exit/quit`
	
	Quit the client
	
	This command will translate to `CORE EXIT`.
* `raw [ARGS]`
	
	send a raw header.
	
	This command does not translate to anything rather send `[ARGS]` as a header.
* `edit <ENTRY ID>`
	
	Grabs text entry and opens it in vim.
	
	This command does not translate to anything rather it makes a `STORAGE GET <ENTRY ID> <COOKIE>` call
	
	then a `STORAGE SET <ENTRY ID> <COOKIE>` call after editing.
