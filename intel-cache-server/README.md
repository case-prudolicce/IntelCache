# The IntelCache Server is the server that will interact with and server the backend Intel (Data) to clients via IcCommands, furnished by IcModules.

It works by translating input commands to corresponding module commands. To view available commands view the [CORE](https://github.com/case-prudolicce/IntelCache/tree/main/intel-cache-lib/ic-core-module) and [STORAGE](https://github.com/case-prudolicce/IntelCache/tree/main/intel-cache-lib/ic-storage-module) READMEs.

# Starting
If starting for the first time, initialize it with `--init`, otherwise just run the program without arguments. Make sure that IPFS is up and working when running the server.

# Arguments
* `--init` will initialize the server.
* `--export` will export the backend SQL database (note, it will not pull large data which is stored in the local IPFS Share).
* `--import` will import the backend SQL databse and use it as a new backend.
* `--teardown` will delete the backend SQL database, it will not delete the stored IPFS data.
* `--testing` will start on a testing port (64290).
* `--testing_export`,`--testing_import`,`--init_testing` and `--testing_teardown` are the same commands ran under the testing port.
