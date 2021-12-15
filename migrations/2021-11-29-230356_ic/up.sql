-- Your SQL goes here
CREATE TABLE dir(
	id INTEGER NOT NULL PRIMARY KEY AUTO_INCREMENT,
	name VARCHAR(50) NOT NULL,
	loc INTEGER NULL,
	CONSTRAINT srk_dirloc FOREIGN KEY (loc) REFERENCES dir (id) 
);

INSERT INTO dir (name) VALUES ("default");

CREATE TABLE entry ( 
	id INTEGER NOT NULL PRIMARY KEY AUTO_INCREMENT, 
	name VARCHAR(50) NOT NULL, 
	data BLOB NOT NULL, 
	type VARCHAR(50) NOT NULL,
	date_added DATETIME NOT NULL DEFAULT NOW(),
	date_last_modified DATETIME NOT NULL DEFAULT NOW(),
	loc INTEGER NOT NULL DEFAULT 1, 
	label TEXT NULL,
	CONSTRAINT fk_entry_d FOREIGN KEY (loc) REFERENCES dir (id)
);

CREATE TABLE tag ( 
	id INTEGER NOT NULL PRIMARY KEY AUTO_INCREMENT, 
	name VARCHAR(50) NOT NULL
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

