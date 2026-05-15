use super::common::{diff_down, diff_up, empty_schema, parse_schema};

#[test]
fn adds_field_index_to_existing_table() {
    let prev = parse_schema(aureline_schema!("table User {", "  email string", "}",));
    let next = parse_schema(aureline_schema!(
        "table User {",
        "  email string @index",
        "}",
    ));

    assert_eq!(
        diff_up(&prev, &next),
        expected_surql!("DEFINE INDEX user_email_idx ON user FIELDS email;")
    );
    assert_eq!(
        diff_down(&prev, &next),
        expected_surql!("REMOVE INDEX user_email_idx ON TABLE user;")
    );
}

#[test]
fn adds_field_index_with_explicit_name() {
    let prev = parse_schema(aureline_schema!("table User {", "  email string", "}",));
    let next = parse_schema(aureline_schema!(
        "table User {",
        "  email string @index(name: email_lookup)",
        "}",
    ));

    assert_eq!(
        diff_up(&prev, &next),
        expected_surql!("DEFINE INDEX email_lookup ON user FIELDS email;")
    );
    assert_eq!(
        diff_down(&prev, &next),
        expected_surql!("REMOVE INDEX email_lookup ON TABLE user;")
    );
}

#[test]
fn adds_composite_index_to_existing_table() {
    let prev = parse_schema(aureline_schema!(
        "table User {",
        "  account string",
        "  email string",
        "}",
    ));
    let next = parse_schema(aureline_schema!(
        "table User {",
        "  account string",
        "  email string",
        "",
        "  @@index(fields: [account, email])",
        "}",
    ));

    assert_eq!(
        diff_up(&prev, &next),
        expected_surql!("DEFINE INDEX user_account_email_idx ON user FIELDS account, email;")
    );
    assert_eq!(
        diff_down(&prev, &next),
        expected_surql!("REMOVE INDEX user_account_email_idx ON TABLE user;")
    );
}

#[test]
fn adds_composite_index_with_explicit_name_to_existing_table() {
    let prev = parse_schema(aureline_schema!(
        "table User {",
        "  account string",
        "  email string",
        "}",
    ));
    let next = parse_schema(aureline_schema!(
        "table User {",
        "  account string",
        "  email string",
        "",
        "  @@index(fields: [account, email], name: account_email_lookup)",
        "}",
    ));

    assert_eq!(
        diff_up(&prev, &next),
        expected_surql!("DEFINE INDEX account_email_lookup ON user FIELDS account, email;")
    );
    assert_eq!(
        diff_down(&prev, &next),
        expected_surql!("REMOVE INDEX account_email_lookup ON TABLE user;")
    );
}

#[test]
fn creates_table_with_indexes_after_fields() {
    let next = parse_schema(aureline_schema!(
        "table User {",
        "  account string",
        "  email string @index",
        "",
        "  @@index(fields: [account, email], name: account_email_lookup)",
        "}",
    ));

    assert_eq!(
        diff_up(&empty_schema(), &next),
        expected_surql!(
            "DEFINE TABLE user;",
            "DEFINE FIELD account ON user TYPE string;",
            "DEFINE FIELD email ON user TYPE string;",
            "DEFINE INDEX account_email_lookup ON user FIELDS account, email;",
            "DEFINE INDEX user_email_idx ON user FIELDS email;",
        )
    );
}

#[test]
fn removes_table_without_removing_its_indexes_separately() {
    let prev = parse_schema(aureline_schema!(
        "table User {",
        "  account string",
        "  email string @index",
        "",
        "  @@index(fields: [account, email], name: account_email_lookup)",
        "}",
    ));

    assert_eq!(
        diff_up(&prev, &empty_schema()),
        expected_surql!("REMOVE TABLE user;")
    );
    assert_eq!(
        diff_down(&prev, &empty_schema()),
        expected_surql!(
            "-- down: RemoveTable User cannot restore data",
            "DEFINE TABLE user;",
        )
    );
}

#[test]
fn removes_field_index_from_existing_table() {
    let prev = parse_schema(aureline_schema!(
        "table User {",
        "  email string @index",
        "}",
    ));
    let next = parse_schema(aureline_schema!("table User {", "  email string", "}",));

    assert_eq!(
        diff_up(&prev, &next),
        expected_surql!("REMOVE INDEX user_email_idx ON TABLE user;")
    );
    assert_eq!(
        diff_down(&prev, &next),
        expected_surql!("DEFINE INDEX user_email_idx ON user FIELDS email;")
    );
}

#[test]
fn removes_composite_index_from_existing_table() {
    let prev = parse_schema(aureline_schema!(
        "table User {",
        "  account string",
        "  email string",
        "",
        "  @@index(fields: [account, email])",
        "}",
    ));
    let next = parse_schema(aureline_schema!(
        "table User {",
        "  account string",
        "  email string",
        "}",
    ));

    assert_eq!(
        diff_up(&prev, &next),
        expected_surql!("REMOVE INDEX user_account_email_idx ON TABLE user;")
    );
    assert_eq!(
        diff_down(&prev, &next),
        expected_surql!("DEFINE INDEX user_account_email_idx ON user FIELDS account, email;")
    );
}

#[test]
fn removes_only_composite_index_when_unique_and_count_remain() {
    let prev = parse_schema(aureline_schema!(
        "table User {",
        "  account string",
        "  email string",
        "",
        "  @@index(fields: [account, email], name: account_email_lookup)",
        "  @@unique(fields: [account, email])",
        "  @@count",
        "}",
    ));
    let next = parse_schema(aureline_schema!(
        "table User {",
        "  account string",
        "  email string",
        "",
        "  @@unique(fields: [account, email])",
        "  @@count",
        "}",
    ));

    assert_eq!(
        diff_up(&prev, &next),
        expected_surql!("REMOVE INDEX account_email_lookup ON TABLE user;")
    );
    assert_eq!(
        diff_down(&prev, &next),
        expected_surql!("DEFINE INDEX account_email_lookup ON user FIELDS account, email;")
    );
}

#[test]
fn renames_index_by_removing_old_index_and_creating_new_index() {
    let prev = parse_schema(aureline_schema!(
        "table User {",
        "  email string @index(name: old_email_lookup)",
        "}",
    ));
    let next = parse_schema(aureline_schema!(
        "table User {",
        "  email string @index(name: new_email_lookup)",
        "}",
    ));

    assert_eq!(
        diff_up(&prev, &next),
        expected_surql!(
            "REMOVE INDEX old_email_lookup ON TABLE user;",
            "DEFINE INDEX new_email_lookup ON user FIELDS email;",
        )
    );
    assert_eq!(
        diff_down(&prev, &next),
        expected_surql!(
            "REMOVE INDEX new_email_lookup ON TABLE user;",
            "DEFINE INDEX old_email_lookup ON user FIELDS email;",
        )
    );
}

#[test]
fn changes_index_fields_by_replacing_the_index() {
    let prev = parse_schema(aureline_schema!(
        "table User {",
        "  account string",
        "  email string",
        "",
        "  @@index(fields: [email], name: user_lookup)",
        "}",
    ));
    let next = parse_schema(aureline_schema!(
        "table User {",
        "  account string",
        "  email string",
        "",
        "  @@index(fields: [account, email], name: user_lookup)",
        "}",
    ));

    assert_eq!(
        diff_up(&prev, &next),
        expected_surql!(
            "REMOVE INDEX user_lookup ON TABLE user;",
            "DEFINE INDEX user_lookup ON user FIELDS account, email;",
        )
    );
    assert_eq!(
        diff_down(&prev, &next),
        expected_surql!(
            "REMOVE INDEX user_lookup ON TABLE user;",
            "DEFINE INDEX user_lookup ON user FIELDS email;",
        )
    );
}

#[test]
fn changes_composite_index_field_order_by_replacing_the_index() {
    let prev = parse_schema(aureline_schema!(
        "table User {",
        "  account string",
        "  email string",
        "",
        "  @@index(fields: [account, email], name: user_lookup)",
        "}",
    ));
    let next = parse_schema(aureline_schema!(
        "table User {",
        "  account string",
        "  email string",
        "",
        "  @@index(fields: [email, account], name: user_lookup)",
        "}",
    ));

    assert_eq!(
        diff_up(&prev, &next),
        expected_surql!(
            "REMOVE INDEX user_lookup ON TABLE user;",
            "DEFINE INDEX user_lookup ON user FIELDS email, account;",
        )
    );
    assert_eq!(
        diff_down(&prev, &next),
        expected_surql!(
            "REMOVE INDEX user_lookup ON TABLE user;",
            "DEFINE INDEX user_lookup ON user FIELDS account, email;",
        )
    );
}

// COUNT indexes are table-level in SurrealDB; they do not have field lists.
#[test]
fn adds_count_index_to_existing_table() {
    let prev = parse_schema(aureline_schema!("table User {", "  email string", "}",));
    let next = parse_schema(aureline_schema!(
        "table User {",
        "  email string",
        "",
        "  @@count",
        "}",
    ));

    assert_eq!(
        diff_up(&prev, &next),
        expected_surql!("DEFINE INDEX user_count ON user COUNT;")
    );
    assert_eq!(
        diff_down(&prev, &next),
        expected_surql!("REMOVE INDEX user_count ON TABLE user;")
    );
}

#[test]
fn removes_count_index_from_existing_table() {
    let prev = parse_schema(aureline_schema!(
        "table User {",
        "  email string",
        "",
        "  @@count",
        "}",
    ));
    let next = parse_schema(aureline_schema!("table User {", "  email string", "}",));

    assert_eq!(
        diff_up(&prev, &next),
        expected_surql!("REMOVE INDEX user_count ON TABLE user;")
    );
    assert_eq!(
        diff_down(&prev, &next),
        expected_surql!("DEFINE INDEX user_count ON user COUNT;")
    );
}

#[test]
fn replaces_count_with_standard_index_as_remove_and_define() {
    let prev = parse_schema(aureline_schema!(
        "table User {",
        "  email string",
        "",
        "  @@count",
        "}",
    ));
    let next = parse_schema(aureline_schema!(
        "table User {",
        "  email string @index",
        "}",
    ));

    assert_eq!(
        diff_up(&prev, &next),
        expected_surql!(
            "REMOVE INDEX user_count ON TABLE user;",
            "DEFINE INDEX user_email_idx ON user FIELDS email;",
        )
    );
    assert_eq!(
        diff_down(&prev, &next),
        expected_surql!(
            "REMOVE INDEX user_email_idx ON TABLE user;",
            "DEFINE INDEX user_count ON user COUNT;",
        )
    );
}

#[test]
fn creates_table_with_standard_unique_and_count_indexes_after_fields() {
    let next = parse_schema(aureline_schema!(
        "table User {",
        "  account string",
        "  email string @index",
        "",
        "  @@index(fields: [account, email], name: account_email_lookup)",
        "  @@unique(fields: [account, email])",
        "  @@count",
        "}",
    ));

    assert_eq!(
        diff_up(&empty_schema(), &next),
        expected_surql!(
            "DEFINE TABLE user;",
            "DEFINE FIELD account ON user TYPE string;",
            "DEFINE FIELD email ON user TYPE string;",
            "DEFINE INDEX account_email_lookup ON user FIELDS account, email;",
            "DEFINE INDEX user_account_email_unique ON user FIELDS account, email UNIQUE;",
            "DEFINE INDEX user_count ON user COUNT;",
            "DEFINE INDEX user_email_idx ON user FIELDS email;",
        )
    );
}

#[test]
fn creates_table_with_count_index_after_fields() {
    let next = parse_schema(aureline_schema!(
        "table User {",
        "  email string",
        "",
        "  @@count",
        "}",
    ));

    assert_eq!(
        diff_up(&empty_schema(), &next),
        expected_surql!(
            "DEFINE TABLE user;",
            "DEFINE FIELD email ON user TYPE string;",
            "DEFINE INDEX user_count ON user COUNT;",
        )
    );
}
