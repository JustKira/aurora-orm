use crate::ast::{AttributeArg, AttributeValue, Schema, SchemaItem};

use super::super::SemanticError;
use super::super::diagnostics::unknown_surql_variable;

pub(super) fn analyze(schema: &Schema, errors: &mut Vec<SemanticError>) {
    for item in &schema.items {
        match item {
            SchemaItem::TableDecl(table) => {
                for field in &table.fields {
                    for attr in &field.raw_attributes {
                        match attr.name.as_str() {
                            "assert" => assert_attribute(attr.args.as_slice(), errors),
                            "allow" => allow_attribute(attr.args.as_slice(), errors),
                            _ => {}
                        }
                    }
                }
            }
            SchemaItem::DocComment { .. }
            | SchemaItem::AnalyzerDecl(_)
            | SchemaItem::FunctionDecl(_) => {}
        }
    }
}

fn assert_attribute(args: &[AttributeArg], errors: &mut Vec<SemanticError>) {
    // `@assert` is a field-value expression. SurrealDB exposes the asserted
    // value, the input value, and the current record in this context.
    let Some((body, range)) = only_surql_arg(args) else {
        return;
    };
    variable_scope(body, &["value", "input", "this"], range, errors);
}

fn allow_attribute(args: &[AttributeArg], errors: &mut Vec<SemanticError>) {
    // `@allow` uses SurrealDB permission syntax (`WHERE ...`). Aureline only
    // exposes auth context here; field-value checks belong in `@assert`.
    let Some((body, range)) = allow_surql_arg(args) else {
        return;
    };
    variable_scope(body, &["auth"], range, errors);
}

fn only_surql_arg(args: &[AttributeArg]) -> Option<(&str, Option<crate::SourceRange>)> {
    // If the argument shape is wrong, the attribute parser will report that.
    // This pass only checks variable scope for an otherwise valid SurQL block.
    let [
        AttributeArg::Positional {
            value: AttributeValue::Surql { body, source_range },
        },
    ] = args
    else {
        return None;
    };
    Some((body.as_str(), *source_range))
}

fn allow_surql_arg(args: &[AttributeArg]) -> Option<(&str, Option<crate::SourceRange>)> {
    // If the `@allow` shape is invalid, lowering reports the argument error.
    // This pass only scope-checks variables once there is exactly one operation
    // and one permission block to analyze.
    let mut has_operation = false;
    let mut permission = None;

    for arg in args {
        match arg {
            AttributeArg::Keyword {
                name,
                value: AttributeValue::String { .. },
            } if name == "op" => {
                if has_operation {
                    return None;
                }
                has_operation = true;
            }
            AttributeArg::Positional {
                value: AttributeValue::Surql { body, source_range },
            } => {
                if permission.is_some() {
                    return None;
                }
                permission = Some((body.as_str(), *source_range));
            }
            _ => return None,
        }
    }

    has_operation.then_some(permission).flatten()
}

fn variable_scope(
    body: &str,
    allowed: &[&str],
    range: Option<crate::SourceRange>,
    errors: &mut Vec<SemanticError>,
) {
    for variable in unknown_variables(body, allowed) {
        errors.push(unknown_surql_variable(variable).at(range));
    }
}

fn unknown_variables(body: &str, allowed: &[&str]) -> Vec<String> {
    // SurrealDB parses into an AST first, but in `surrealdb-core` 3.1 the
    // useful expression nodes (`Expr`, `TopLevelExpr`) are crate-private.
    // Until the public API exposes a visitor or parameter iterator, this
    // lexical scan is the smallest scoped fallback for field-level escape
    // hatches. It handles closure params like `|$val|`; future function bodies
    // with `LET $c = ...` will need a fuller scoped analyzer.
    let mut scanner = VariableScanner::new(body, allowed);
    scanner.scan()
}

struct VariableScanner<'a, 'b> {
    input: &'a str,
    pos: usize,
    depth: usize,
    allowed: &'b [&'b str],
    scopes: Vec<VariableScope>,
    unknown: Vec<String>,
}

struct VariableScope {
    names: Vec<String>,
    depth: usize,
}

impl<'a, 'b> VariableScanner<'a, 'b> {
    fn new(input: &'a str, allowed: &'b [&'b str]) -> Self {
        Self {
            input,
            pos: 0,
            depth: 0,
            allowed,
            scopes: Vec::new(),
            unknown: Vec::new(),
        }
    }

    fn scan(&mut self) -> Vec<String> {
        while let Some(ch) = self.peek_char() {
            match ch {
                // Do not treat `$name` inside strings or comments as a variable
                // reference used by the query.
                '\'' | '"' | '`' => {
                    self.advance_char();
                    self.skip_quoted(ch);
                }
                '/' if self.peek_next_char() == Some('/') => self.skip_line_comment(),
                '/' if self.peek_next_char() == Some('*') => self.skip_block_comment(),
                '(' | '[' | '{' => {
                    self.advance_char();
                    self.depth += 1;
                }
                ')' | ']' | '}' => {
                    self.pop_scopes_ending_at_current_depth();
                    self.advance_char();
                    self.depth = self.depth.saturating_sub(1);
                }
                ',' => {
                    self.pop_scopes_ending_at_current_depth();
                    self.advance_char();
                }
                '|' if self.try_read_closure_params() => {}
                '$' => {
                    self.advance_char();
                    self.read_variable();
                }
                _ => {
                    self.advance_char();
                }
            }
        }

        std::mem::take(&mut self.unknown)
    }

    fn read_variable(&mut self) {
        let Some(first) = self.peek_char() else {
            return;
        };
        if !is_identifier_start(first) {
            return;
        }

        let start = self.pos;
        self.advance_char();
        while let Some(ch) = self.peek_char() {
            if !is_identifier_continue(ch) {
                break;
            }
            self.advance_char();
        }

        let name = self.input[start..self.pos].to_string();
        if !self.is_known_variable(&name) {
            self.unknown.push(name);
        }
    }

    fn try_read_closure_params(&mut self) -> bool {
        let Some((names, end)) = parse_closure_params(self.input, self.pos) else {
            return false;
        };

        self.pos = end;
        self.scopes.push(VariableScope {
            names,
            depth: self.depth,
        });
        true
    }

    fn is_known_variable(&self, name: &str) -> bool {
        self.allowed.contains(&name)
            || self
                .scopes
                .iter()
                .rev()
                .any(|scope| scope.names.iter().any(|local| local == name))
    }

    fn pop_scopes_ending_at_current_depth(&mut self) {
        while matches!(self.scopes.last(), Some(scope) if scope.depth == self.depth) {
            self.scopes.pop();
        }
    }

    fn skip_quoted(&mut self, quote: char) {
        let mut escaped = false;
        while let Some(ch) = self.advance_char() {
            if escaped {
                escaped = false;
                continue;
            }
            if ch == '\\' {
                escaped = true;
                continue;
            }
            if ch == quote {
                break;
            }
        }
    }

    fn skip_line_comment(&mut self) {
        self.advance_char();
        self.advance_char();
        while let Some(ch) = self.advance_char() {
            if ch == '\n' {
                break;
            }
        }
    }

    fn skip_block_comment(&mut self) {
        self.advance_char();
        self.advance_char();
        let mut previous = '\0';
        while let Some(ch) = self.advance_char() {
            if previous == '*' && ch == '/' {
                break;
            }
            previous = ch;
        }
    }

    fn peek_char(&self) -> Option<char> {
        self.input[self.pos..].chars().next()
    }

    fn peek_next_char(&self) -> Option<char> {
        let mut chars = self.input[self.pos..].chars();
        chars.next()?;
        chars.next()
    }

    fn advance_char(&mut self) -> Option<char> {
        let ch = self.peek_char()?;
        self.pos += ch.len_utf8();
        Some(ch)
    }
}

fn parse_closure_params(input: &str, start: usize) -> Option<(Vec<String>, usize)> {
    let mut pos = start;
    if char_at(input, pos)? != '|' {
        return None;
    }
    advance_at(input, &mut pos);

    let mut names = Vec::new();
    loop {
        skip_ascii_whitespace(input, &mut pos);

        if char_at(input, pos)? == '|' {
            advance_at(input, &mut pos);
            return Some((names, pos));
        }

        if char_at(input, pos)? != '$' {
            return None;
        }
        advance_at(input, &mut pos);

        let name_start = pos;
        let first = char_at(input, pos)?;
        if !is_identifier_start(first) {
            return None;
        }
        advance_at(input, &mut pos);
        while let Some(ch) = char_at(input, pos) {
            if !is_identifier_continue(ch) {
                break;
            }
            advance_at(input, &mut pos);
        }
        names.push(input[name_start..pos].to_string());

        skip_ascii_whitespace(input, &mut pos);
        match char_at(input, pos)? {
            ',' => {
                advance_at(input, &mut pos);
            }
            '|' => {
                advance_at(input, &mut pos);
                return Some((names, pos));
            }
            _ => return None,
        }
    }
}

fn skip_ascii_whitespace(input: &str, pos: &mut usize) {
    while matches!(char_at(input, *pos), Some(ch) if ch.is_ascii_whitespace()) {
        advance_at(input, pos);
    }
}

fn char_at(input: &str, pos: usize) -> Option<char> {
    input.get(pos..)?.chars().next()
}

fn advance_at(input: &str, pos: &mut usize) -> Option<char> {
    let ch = char_at(input, *pos)?;
    *pos += ch.len_utf8();
    Some(ch)
}

fn is_identifier_start(ch: char) -> bool {
    ch == '_' || ch.is_ascii_alphabetic()
}

fn is_identifier_continue(ch: char) -> bool {
    is_identifier_start(ch) || ch.is_ascii_digit()
}
