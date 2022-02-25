-- Your SQL goes here
CREATE TABLE user(
	id INTEGER NOT NULL PRIMARY KEY AUTO_INCREMENT,
	global_id VARCHAR(128) NOT NULL UNIQUE,
	username VARCHAR(50) NOT NULL,
	password VARCHAR(128) NOT NULL,
	admin BOOLEAN NOT NULL DEFAULT FALSE
);

-- default login: admin:admin, global_id remains the same throughout all the instances.
INSERT INTO user(global_id,username,password,admin) VALUES ("e3351b7f8bbd01daf63a2978657c50faf30f6301ba2bb32293c85576a5afd003","admin","c7ad44cbad762a5da0a452f9e854fdc1e0e7a52a38015f23f3eab1d80b931dd472634dfac71cd34ebc35d16ab7fb8a90c81f975113d6c7538dc69dd8de9077ec",TRUE);

CREATE TABLE dir(
	id INTEGER NOT NULL PRIMARY KEY AUTO_INCREMENT,
	name VARCHAR(50) NOT NULL,
	loc INTEGER NULL,
	visibility BOOLEAN NOT NULL DEFAULT FALSE,
	owner VARCHAR(128) NOT NULL,
	CONSTRAINT fk_dirloc FOREIGN KEY (loc) REFERENCES dir (id) ON DELETE CASCADE,
	CONSTRAINT fk_dir_owner FOREIGN KEY (owner) REFERENCES user(global_id) ON DELETE CASCADE
);

CREATE TABLE entry ( 
	id INTEGER NOT NULL PRIMARY KEY AUTO_INCREMENT, 
	name VARCHAR(50) NOT NULL, 
	data BLOB NOT NULL, 
	type VARCHAR(50) NOT NULL,
	date_added DATETIME NOT NULL DEFAULT NOW(),
	date_last_modified DATETIME NOT NULL DEFAULT NOW(),
	loc INTEGER NULL, 
	label TEXT NULL,
	visibility BOOLEAN NOT NULL DEFAULT FALSE,
	owner varchar(128) NOT NULL,
	CONSTRAINT fk_entry_d FOREIGN KEY (loc) REFERENCES dir (id) ON DELETE CASCADE,
	CONSTRAINT fk_entry_owner FOREIGN KEY (owner) REFERENCES user (global_id) ON DELETE CASCADE
);

CREATE TABLE tag ( 
	id INTEGER NOT NULL PRIMARY KEY AUTO_INCREMENT, 
	name VARCHAR(50) NOT NULL,
	owner VARCHAR(128) NOT NULL,
	visibility BOOLEAN NOT NULL DEFAULT FALSE,
	CONSTRAINT fk_tag_owner FOREIGN KEY (owner) REFERENCES user (global_id) ON DELETE CASCADE
);

CREATE TABLE entry_tags ( 
	entryid INTEGER NOT NULL, 
	tagid INTEGER NOT NULL,
	CONSTRAINT fk_etags_e FOREIGN KEY (entryid) REFERENCES entry (id) ON DELETE CASCADE,
	CONSTRAINT fk_etags_t FOREIGN KEY (tagid) REFERENCES tag (id) ON DELETE CASCADE,
	CONSTRAINT pk_etags PRIMARY KEY (entryid,tagid)
);

CREATE TABLE dir_tags ( 
	dirid INTEGER NOT NULL, 
	tagid INTEGER NOT NULL,
	CONSTRAINT fk_dtags_d FOREIGN KEY (dirid) REFERENCES dir (id) ON DELETE CASCADE,
	CONSTRAINT fk_dtags_t FOREIGN KEY (tagid) REFERENCES tag (id) ON DELETE CASCADE,
	CONSTRAINT pk_dtags PRIMARY KEY (dirid,tagid)
);

CREATE TABLE links (
	linkname VARCHAR(200) NOT NULL,
	eid INTEGER NOT NULL,
	CONSTRAINT fk_links_e FOREIGN KEY (eid) REFERENCES entry (id) ON DELETE CASCADE,
	CONSTRAINT pk_links PRIMARY KEY (linkname,eid) 
);

