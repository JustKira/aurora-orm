; Focused Zed highlights for surrealdb/surrealql-tree-sitter.

[
  (keyword_access)
  (keyword_after)
  (keyword_algorithm)
  (keyword_all)
  (keyword_alter)
  (keyword_analyzer)
  (keyword_and)
  (keyword_any)
  (keyword_as)
  (keyword_asc)
  (keyword_assert)
  (keyword_async)
  (keyword_before)
  (keyword_begin)
  (keyword_bm25)
  (keyword_break)
  (keyword_by)
  (keyword_cancel)
  (keyword_cascade)
  (keyword_changefeed)
  (keyword_collate)
  (keyword_columns)
  (keyword_comment)
  (keyword_commit)
  (keyword_computed)
  (keyword_contains)
  (keyword_content)
  (keyword_continue)
  (keyword_count)
  (keyword_create)
  (keyword_database)
  (keyword_db)
  (keyword_default)
  (keyword_define)
  (keyword_delete)
  (keyword_desc)
  (keyword_diff)
  (keyword_dimension)
  (keyword_dist)
  (keyword_drop)
  (keyword_else)
  (keyword_end)
  (keyword_event)
  (keyword_exists)
  (keyword_explain)
  (keyword_fetch)
  (keyword_field)
  (keyword_fields)
  (keyword_filters)
  (keyword_flexible)
  (keyword_for)
  (keyword_from)
  (keyword_function)
  (keyword_group)
  (keyword_highlights)
  (keyword_hnsw)
  (keyword_if)
  (keyword_in)
  (keyword_index)
  (keyword_info)
  (keyword_insert)
  (keyword_into)
  (keyword_is)
  (keyword_let)
  (keyword_limit)
  (keyword_live)
  (keyword_merge)
  (keyword_namespace)
  (keyword_not)
  (keyword_null)
  (keyword_on)
  (keyword_only)
  (keyword_or)
  (keyword_order)
  (keyword_overwrite)
  (keyword_parallel)
  (keyword_param)
  (keyword_permissions)
  (keyword_record)
  (keyword_relate)
  (keyword_remove)
  (keyword_return)
  (keyword_schemafull)
  (keyword_schemaless)
  (keyword_search)
  (keyword_select)
  (keyword_set)
  (keyword_show)
  (keyword_signup)
  (keyword_signin)
  (keyword_sleep)
  (keyword_table)
  (keyword_then)
  (keyword_throw)
  (keyword_timeout)
  (keyword_to)
  (keyword_tokenizers)
  (keyword_transaction)
  (keyword_type)
  (keyword_unique)
  (keyword_unset)
  (keyword_update)
  (keyword_upsert)
  (keyword_use)
  (keyword_value)
  (keyword_values)
  (keyword_when)
  (keyword_where)
  (keyword_with)
] @keyword

(keyword_let) @keyword.storage
(keyword_function) @keyword.function
(keyword_async) @keyword.coroutine
[
  (keyword_if)
  (keyword_else)
  (keyword_then)
  (keyword_end)
] @keyword.control.conditional

[
  (keyword_true)
  (keyword_false)
] @boolean
[
  (keyword_none)
  (keyword_null)
] @constant.builtin

[
  (string)
  (prefixed_string)
] @string
[
  (int)
  (float)
  (decimal)
  (duration)
] @number
(comment) @comment

(function_call (builtin_function_name) @function.builtin)
(function_call (custom_function_name) @function)
(function_call (function_name) @function)
(builtin_function_name) @constant.builtin
(custom_function_name) @function
(variable_name) @variable.parameter
(identifier) @variable

(object_property (object_key) @property)
(field_assignment (identifier) @property)
[
  (type)
  (type_name)
  (parameterized_type)
] @type
(record_id) @variable.special

[
  (binary_operator)
  (operator)
  (graph_path)
] @operator
[
  "("
  ")"
  "["
  "]"
  "<"
  ">"
  "{"
  "}"
] @punctuation.bracket
[
  ","
  ":"
] @punctuation.delimiter
"=" @operator
