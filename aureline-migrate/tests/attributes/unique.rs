use super::common::{diff_down, diff_up, empty_schema, parse_schema};

#[test]
fn adds_field_unique_to_existing_table() {
    let prev = parse_schema(aureline_schema!("table User {", "  email string", "}",));
    let next = parse_schema(aureline_schema!(
        "table User {",
        "  email string @unique",
        "}",
    ));

    assert_eq!(
        diff_up(&prev, &next),
        expected_surql!("DEFINE INDEX user_email_unique ON user FIELDS email UNIQUE;")
    );
    assert_eq!(
        diff_down(&prev, &next),
        expected_surql!("REMOVE INDEX user_email_unique ON TABLE user;")
    );
}

#[test]
fn adds_field_unique_with_explicit_name() {
    let prev = parse_schema(aureline_schema!("table User {", "  email string", "}",));
    let next = parse_schema(aureline_schema!(
        "table User {",
        "  email string @unique(name: unique_email)",
        "}",
    ));

    assert_eq!(
        diff_up(&prev, &next),
        expected_surql!("DEFINE INDEX unique_email ON user FIELDS email UNIQUE;")
    );
    assert_eq!(
        diff_down(&prev, &next),
        expected_surql!("REMOVE INDEX unique_email ON TABLE user;")
    );
}

#[test]
fn adds_composite_unique_to_existing_table() {
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
        "  @@unique(fields: [account, email])",
        "}",
    ));

    assert_eq!(
        diff_up(&prev, &next),
        expected_surql!(
            "DEFINE INDEX user_account_email_unique ON user FIELDS account, email UNIQUE;"
        )
    );
    assert_eq!(
        diff_down(&prev, &next),
        expected_surql!("REMOVE INDEX user_account_email_unique ON TABLE user;")
    );
}

#[test]
fn adds_composite_unique_with_explicit_name_to_existing_table() {
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
        "  @@unique(fields: [account, email], name: account_email_unique)",
        "}",
    ));

    assert_eq!(
        diff_up(&prev, &next),
        expected_surql!("DEFINE INDEX account_email_unique ON user FIELDS account, email UNIQUE;")
    );
    assert_eq!(
        diff_down(&prev, &next),
        expected_surql!("REMOVE INDEX account_email_unique ON TABLE user;")
    );
}

#[test]
fn creates_table_with_uniques_after_fields() {
    let next = parse_schema(aureline_schema!(
        "table User {",
        "  account string",
        "  email string @unique",
        "",
        "  @@unique(fields: [account, email], name: account_email_unique)",
        "}",
    ));

    assert_eq!(
        diff_up(&empty_schema(), &next),
        expected_surql!(
            "DEFINE TABLE user;",
            "DEFINE FIELD account ON user TYPE string;",
            "DEFINE FIELD email ON user TYPE string;",
            "DEFINE INDEX account_email_unique ON user FIELDS account, email UNIQUE;",
            "DEFINE INDEX user_email_unique ON user FIELDS email UNIQUE;",
        )
    );
}

#[test]
fn removes_field_unique_from_existing_table() {
    let prev = parse_schema(aureline_schema!(
        "table User {",
        "  email string @unique",
        "}",
    ));
    let next = parse_schema(aureline_schema!("table User {", "  email string", "}",));

    assert_eq!(
        diff_up(&prev, &next),
        expected_surql!("REMOVE INDEX user_email_unique ON TABLE user;")
    );
    assert_eq!(
        diff_down(&prev, &next),
        expected_surql!("DEFINE INDEX user_email_unique ON user FIELDS email UNIQUE;")
    );
}

#[test]
fn removes_composite_unique_from_existing_table() {
    let prev = parse_schema(aureline_schema!(
        "table User {",
        "  account string",
        "  email string",
        "",
        "  @@unique(fields: [account, email])",
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
        expected_surql!("REMOVE INDEX user_account_email_unique ON TABLE user;")
    );
    assert_eq!(
        diff_down(&prev, &next),
        expected_surql!(
            "DEFINE INDEX user_account_email_unique ON user FIELDS account, email UNIQUE;"
        )
    );
}

#[test]
fn removes_only_composite_unique_when_standard_and_count_remain() {
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
        "  @@index(fields: [account, email], name: account_email_lookup)",
        "  @@count",
        "}",
    ));

    assert_eq!(
        diff_up(&prev, &next),
        expected_surql!("REMOVE INDEX user_account_email_unique ON TABLE user;")
    );
    assert_eq!(
        diff_down(&prev, &next),
        expected_surql!(
            "DEFINE INDEX user_account_email_unique ON user FIELDS account, email UNIQUE;"
        )
    );
}

#[test]
fn renames_unique_by_removing_old_unique_and_creating_new_unique() {
    let prev = parse_schema(aureline_schema!(
        "table User {",
        "  email string @unique(name: old_unique_email)",
        "}",
    ));
    let next = parse_schema(aureline_schema!(
        "table User {",
        "  email string @unique(name: new_unique_email)",
        "}",
    ));

    assert_eq!(
        diff_up(&prev, &next),
        expected_surql!(
            "REMOVE INDEX old_unique_email ON TABLE user;",
            "DEFINE INDEX new_unique_email ON user FIELDS email UNIQUE;",
        )
    );
    assert_eq!(
        diff_down(&prev, &next),
        expected_surql!(
            "REMOVE INDEX new_unique_email ON TABLE user;",
            "DEFINE INDEX old_unique_email ON user FIELDS email UNIQUE;",
        )
    );
}

#[test]
fn changes_unique_fields_by_replacing_the_unique_index() {
    let prev = parse_schema(aureline_schema!(
        "table User {",
        "  account string",
        "  email string",
        "",
        "  @@unique(fields: [email], name: user_unique)",
        "}",
    ));
    let next = parse_schema(aureline_schema!(
        "table User {",
        "  account string",
        "  email string",
        "",
        "  @@unique(fields: [account, email], name: user_unique)",
        "}",
    ));

    assert_eq!(
        diff_up(&prev, &next),
        expected_surql!(
            "REMOVE INDEX user_unique ON TABLE user;",
            "DEFINE INDEX user_unique ON user FIELDS account, email UNIQUE;",
        )
    );
    assert_eq!(
        diff_down(&prev, &next),
        expected_surql!(
            "REMOVE INDEX user_unique ON TABLE user;",
            "DEFINE INDEX user_unique ON user FIELDS email UNIQUE;",
        )
    );
}

#[test]
fn changes_composite_unique_field_order_by_replacing_the_unique_index() {
    let prev = parse_schema(aureline_schema!(
        "table User {",
        "  account string",
        "  email string",
        "",
        "  @@unique(fields: [account, email], name: user_unique)",
        "}",
    ));
    let next = parse_schema(aureline_schema!(
        "table User {",
        "  account string",
        "  email string",
        "",
        "  @@unique(fields: [email, account], name: user_unique)",
        "}",
    ));

    assert_eq!(
        diff_up(&prev, &next),
        expected_surql!(
            "REMOVE INDEX user_unique ON TABLE user;",
            "DEFINE INDEX user_unique ON user FIELDS email, account UNIQUE;",
        )
    );
    assert_eq!(
        diff_down(&prev, &next),
        expected_surql!(
            "REMOVE INDEX user_unique ON TABLE user;",
            "DEFINE INDEX user_unique ON user FIELDS account, email UNIQUE;",
        )
    );
}

#[test]
fn changing_standard_index_to_unique_replaces_the_index_kind() {
    let prev = parse_schema(aureline_schema!(
        "table User {",
        "  email string @index(name: email_constraint)",
        "}",
    ));
    let next = parse_schema(aureline_schema!(
        "table User {",
        "  email string @unique(name: email_constraint)",
        "}",
    ));

    assert_eq!(
        diff_up(&prev, &next),
        expected_surql!(
            "REMOVE INDEX email_constraint ON TABLE user;",
            "DEFINE INDEX email_constraint ON user FIELDS email UNIQUE;",
        )
    );
    assert_eq!(
        diff_down(&prev, &next),
        expected_surql!(
            "REMOVE INDEX email_constraint ON TABLE user;",
            "DEFINE INDEX email_constraint ON user FIELDS email;",
        )
    );
}
