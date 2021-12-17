table! {
    dir (id) {
        id -> Integer,
        name -> Varchar,
        loc -> Nullable<Integer>,
    }
}

table! {
    dir_tags (dirid, tagid) {
        dirid -> Integer,
        tagid -> Integer,
    }
}

table! {
    entry (id) {
        id -> Integer,
        name -> Varchar,
        data -> Blob,
        #[sql_name = "type"]
        type_ -> Varchar,
        date_added -> Datetime,
        date_last_modified -> Datetime,
        loc -> Integer,
        label -> Nullable<Text>,
    }
}

table! {
    entry_tags (entryid, tagid) {
        entryid -> Integer,
        tagid -> Integer,
    }
}

table! {
    links (linkname, eid) {
        linkname -> Varchar,
        eid -> Integer,
    }
}

table! {
    tag (id) {
        id -> Integer,
        name -> Varchar,
    }
}

joinable!(dir_tags -> dir (dirid));
joinable!(dir_tags -> tag (tagid));
joinable!(entry -> dir (loc));
joinable!(entry_tags -> entry (entryid));
joinable!(entry_tags -> tag (tagid));
joinable!(links -> entry (eid));

allow_tables_to_appear_in_same_query!(
    dir,
    dir_tags,
    entry,
    entry_tags,
    links,
    tag,
);
