table! {
    dir (id) {
        id -> Integer,
        name -> Varchar,
        loc -> Nullable<Integer>,
        visibility -> Bool,
        owner -> Varchar,
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
        loc -> Nullable<Integer>,
        label -> Nullable<Text>,
        visibility -> Bool,
        owner -> Varchar,
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
        owner -> Varchar,
        visibility -> Bool,
    }
}

table! {
    user (id) {
        id -> Integer,
        global_id -> Varchar,
        username -> Varchar,
        password -> Varchar,
        admin -> Bool,
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
    user,
);
