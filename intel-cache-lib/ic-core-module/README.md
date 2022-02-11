# The CORE module deals with accounts

## Commands
* ```CORE LOGIN <GLOBAL ID> <PASSWORD>```

The password must be hashed with sha512

Returns the cookie for the session in the header if valid.
* ```CORE REGISTER <USERNAME> <PASSWORD>```

The password must be hashed with sha512

Returns OK! in the header if successfull
* ```CORE FETCH USERS <USERNAME>```

Returns `GlOBAL ID`s from the given username (in the body), and `UNIQUE` or `MULTIPLE` in the header if found.
* ```CORE NULL```

(Deprecated) used as heartbeat/alive signals to the server. It will return an empty response if connected.

