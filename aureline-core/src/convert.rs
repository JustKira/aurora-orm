use pest::Span;
use pest_ast::FromPest;

use crate::ast;
use crate::check::diagnostics::{SourcePosition, SourceRange};
use crate::grammar::Rule;

fn span_to_string(span: Span<'_>) -> String {
    span.as_str().to_string()
}

fn span_to_source_range(span: Span<'_>) -> SourceRange {
    let (start_line, start_column) = span.start_pos().line_col();
    let (end_line, end_column) = span.end_pos().line_col();
    SourceRange {
        start: SourcePosition {
            line: start_line.saturating_sub(1) as u32,
            character: start_column.saturating_sub(1) as u32,
        },
        end: SourcePosition {
            line: end_line.saturating_sub(1) as u32,
            character: end_column.saturating_sub(1) as u32,
        },
    }
}

#[derive(FromPest)]
#[pest_ast(rule(Rule::schema))]
pub struct Schema {
    pub items: Vec<SchemaItem>,
    _eoi: Eoi,
}

#[derive(FromPest)]
#[pest_ast(rule(Rule::source_file))]
pub struct SourceFile {
    pub items: Vec<SourceItem>,
    _eoi: Eoi,
}

#[derive(FromPest)]
#[pest_ast(rule(Rule::schema_item))]
pub enum SchemaItem {
    DocComment(DocComment),
    TableBlock(TableBlock),
    AnalyzerBlock(AnalyzerBlock),
    FunctionBlock(FunctionBlock),
}

#[derive(FromPest)]
#[pest_ast(rule(Rule::source_items))]
pub enum SourceItem {
    DocComment(DocComment),
    TableBlock(TableBlock),
    AnalyzerBlock(AnalyzerBlock),
    FunctionBlock(FunctionBlock),
    InvalidLine(InvalidLine),
}

#[derive(FromPest)]
#[pest_ast(rule(Rule::INVALID_SOURCE_ITEM))]
pub struct InvalidLine;

#[derive(FromPest)]
#[pest_ast(rule(Rule::surql_block))]
#[allow(dead_code)]
pub struct SurqlBlock {
    #[pest_ast(outer(with(span_to_source_range)))]
    pub source_range: SourceRange,
    #[pest_ast(outer(with(span_to_string)))]
    pub source: String,
    pub chunks: Vec<SurqlChunk>,
}

#[derive(FromPest)]
#[pest_ast(rule(Rule::surql_inline))]
pub struct SurqlInline {
    #[pest_ast(outer(with(span_to_source_range)))]
    pub source_range: SourceRange,
    #[pest_ast(outer(with(span_to_string)))]
    pub source: String,
}

#[derive(FromPest)]
#[pest_ast(rule(Rule::surql_chunk))]
#[allow(dead_code)]
pub enum SurqlChunk {
    Nested(SurqlNested),
    String(SurqlString),
    Comment(SurqlComment),
    Text(SurqlText),
}

#[derive(FromPest)]
#[pest_ast(rule(Rule::surql_nested))]
#[allow(dead_code)]
pub struct SurqlNested {
    pub chunks: Vec<SurqlChunk>,
}

#[derive(FromPest)]
#[pest_ast(rule(Rule::surql_string))]
#[allow(dead_code)]
pub struct SurqlString {
    #[pest_ast(outer(with(span_to_string)))]
    pub text: String,
}

#[derive(FromPest)]
#[pest_ast(rule(Rule::surql_comment))]
#[allow(dead_code)]
pub struct SurqlComment {
    #[pest_ast(outer(with(span_to_string)))]
    pub text: String,
}

#[derive(FromPest)]
#[pest_ast(rule(Rule::surql_text))]
#[allow(dead_code)]
pub struct SurqlText {
    #[pest_ast(outer(with(span_to_string)))]
    pub text: String,
}

#[derive(FromPest)]
#[pest_ast(rule(Rule::doc_comment))]
pub struct DocComment {
    pub lines: Vec<DocCommentLine>,
}

#[derive(FromPest)]
#[pest_ast(rule(Rule::doc_comment_line))]
pub struct DocCommentLine {
    #[pest_ast(outer(with(span_to_string)))]
    pub content: String,
}

#[derive(FromPest)]
#[pest_ast(rule(Rule::table_block))]
pub struct TableBlock {
    #[pest_ast(outer(with(span_to_source_range)))]
    pub source_range: SourceRange,
    pub name: Identifier,
    pub modifier: Option<TableModifier>,
    pub members: Vec<TableMember>,
}

#[derive(FromPest)]
#[pest_ast(rule(Rule::table_modifier))]
pub struct TableModifier {
    #[pest_ast(outer(with(span_to_string)))]
    pub value: String,
}

#[derive(FromPest)]
#[pest_ast(rule(Rule::table_member))]
pub enum TableMember {
    BlockAttributeLine(BlockAttributeLine),
    FieldLine(FieldLine),
}

#[derive(FromPest)]
#[pest_ast(rule(Rule::field_line))]
pub struct FieldLine {
    pub field: FieldNode,
}

#[derive(FromPest)]
#[pest_ast(rule(Rule::field_attribute_line))]
pub struct FieldAttributeLine {
    pub attribute: AttributeNode,
}

#[derive(FromPest)]
#[pest_ast(rule(Rule::field_attribute_block))]
pub struct FieldAttributeBlock {
    pub attributes: Vec<FieldAttributeLine>,
}

#[derive(FromPest)]
#[pest_ast(rule(Rule::block_attribute_line))]
pub struct BlockAttributeLine {
    pub attribute: BlockAttribute,
}

#[derive(FromPest)]
#[pest_ast(rule(Rule::field))]
pub struct FieldNode {
    #[pest_ast(outer(with(span_to_source_range)))]
    pub source_range: SourceRange,
    pub name: Identifier,
    pub type_expr: TypeExpr,
    pub attributes: Vec<AttributeNode>,
    pub attribute_block: Option<FieldAttributeBlock>,
}

#[derive(FromPest)]
#[pest_ast(rule(Rule::type_expr))]
pub struct TypeExpr {
    pub type_node: TypeNode,
    pub optional: Option<OptionalMarker>,
}

#[derive(FromPest)]
#[pest_ast(rule(Rule::type_node))]
#[allow(clippy::enum_variant_names)]
pub enum TypeNode {
    OptionType(OptionType),
    ArrayType(ArrayType),
    SetType(SetType),
    RecordType(RecordType),
    GeometryType(GeometryType),
    PrimitiveType(PrimitiveType),
}

#[derive(FromPest)]
#[pest_ast(rule(Rule::option_type))]
pub struct OptionType {
    pub inner: Box<TypeNode>,
}

#[derive(FromPest)]
#[pest_ast(rule(Rule::array_type))]
pub struct ArrayType {
    pub inner: Box<TypeNode>,
    pub length: Option<ArrayLength>,
}

#[derive(FromPest)]
#[pest_ast(rule(Rule::set_type))]
pub struct SetType {
    pub inner: Box<TypeNode>,
    pub length: Option<ArrayLength>,
}

#[derive(FromPest)]
#[pest_ast(rule(Rule::record_type))]
pub struct RecordType {
    pub table: Option<Identifier>,
}

#[derive(FromPest)]
#[pest_ast(rule(Rule::geometry_type))]
pub struct GeometryType {
    pub features: Vec<Identifier>,
}

#[derive(FromPest)]
#[pest_ast(rule(Rule::primitive_type))]
pub struct PrimitiveType {
    #[pest_ast(outer(with(span_to_string)))]
    pub value: String,
}

#[derive(FromPest)]
#[pest_ast(rule(Rule::array_length))]
pub struct ArrayLength {
    #[pest_ast(outer(with(span_to_string)))]
    pub digits: String,
}

#[derive(FromPest)]
#[pest_ast(rule(Rule::optional_marker))]
pub struct OptionalMarker;

// === Attributes ===

#[derive(FromPest)]
#[pest_ast(rule(Rule::attribute))]
pub struct AttributeNode {
    #[pest_ast(outer(with(span_to_source_range)))]
    pub source_range: SourceRange,
    pub name: AttributeName,
    pub call: Option<AttrCall>,
}

#[derive(FromPest)]
#[pest_ast(rule(Rule::block_attribute))]
pub struct BlockAttribute {
    #[pest_ast(outer(with(span_to_source_range)))]
    pub source_range: SourceRange,
    pub name: BlockAttributeName,
    pub call: Option<AttrCall>,
}

#[derive(FromPest)]
#[pest_ast(rule(Rule::attribute_name))]
pub struct AttributeName {
    #[pest_ast(outer(with(span_to_string)))]
    pub value: String,
}

#[derive(FromPest)]
#[pest_ast(rule(Rule::block_attribute_name))]
pub struct BlockAttributeName {
    #[pest_ast(outer(with(span_to_string)))]
    pub value: String,
}

#[derive(FromPest)]
#[pest_ast(rule(Rule::attr_call))]
pub struct AttrCall {
    pub args: Option<AttrArgList>,
}

#[derive(FromPest)]
#[pest_ast(rule(Rule::attr_arg_list))]
pub struct AttrArgList {
    pub args: Vec<AttrArg>,
}

#[derive(FromPest)]
#[pest_ast(rule(Rule::attr_arg))]
pub enum AttrArg {
    Kv(AttrKv),
    Value(AttrValueNode),
}

#[derive(FromPest)]
#[pest_ast(rule(Rule::attr_kv))]
pub struct AttrKv {
    pub name: Identifier,
    pub value: AttrValueNode,
}

#[derive(FromPest)]
#[pest_ast(rule(Rule::attr_value))]
pub enum AttrValueNode {
    Surql(SurqlBlock),
    SurqlInline(SurqlInline),
    Array(AttrArray),
    Tuple(AttrTuple),
    Number(AttrNumber),
    Bool(AttrBool),
    String(AttrString),
    Ident(AttrIdent),
}

#[derive(FromPest)]
#[pest_ast(rule(Rule::attr_tuple))]
pub struct AttrTuple {
    pub values: Vec<AttrValueNode>,
}

#[derive(FromPest)]
#[pest_ast(rule(Rule::attr_array))]
pub struct AttrArray {
    pub values: Vec<AttrValueNode>,
}

#[derive(FromPest)]
#[pest_ast(rule(Rule::attr_number))]
pub struct AttrNumber {
    #[pest_ast(outer(with(span_to_string)))]
    pub value: String,
}

#[derive(FromPest)]
#[pest_ast(rule(Rule::attr_bool))]
pub struct AttrBool {
    #[pest_ast(outer(with(span_to_string)))]
    pub value: String,
}

#[derive(FromPest)]
#[pest_ast(rule(Rule::attr_ident))]
pub struct AttrIdent {
    #[pest_ast(outer(with(span_to_string)))]
    pub value: String,
}

#[derive(FromPest)]
#[pest_ast(rule(Rule::attr_string))]
pub struct AttrString {
    #[pest_ast(outer(with(span_to_string)))]
    pub value: String,
}

// === Analyzer ===

#[derive(FromPest)]
#[pest_ast(rule(Rule::analyzer_block))]
pub struct AnalyzerBlock {
    #[pest_ast(outer(with(span_to_source_range)))]
    pub source_range: SourceRange,
    pub name: Identifier,
    pub members: Vec<AnalyzerMember>,
}

#[derive(FromPest)]
#[pest_ast(rule(Rule::analyzer_member))]
pub enum AnalyzerMember {
    ClauseLine(AnalyzerClauseLine),
}

#[derive(FromPest)]
#[pest_ast(rule(Rule::analyzer_clause_line))]
pub struct AnalyzerClauseLine {
    pub clause: AnalyzerClause,
}

#[derive(FromPest)]
#[pest_ast(rule(Rule::analyzer_clause))]
pub enum AnalyzerClause {
    Tokenizers(AnalyzerTokenizers),
    Filters(AnalyzerFilters),
}

#[derive(FromPest)]
#[pest_ast(rule(Rule::analyzer_tokenizers))]
pub struct AnalyzerTokenizers {
    pub names: Vec<Identifier>,
}

#[derive(FromPest)]
#[pest_ast(rule(Rule::analyzer_filters))]
pub struct AnalyzerFilters {
    pub calls: Vec<FilterCallNode>,
}

#[derive(FromPest)]
#[pest_ast(rule(Rule::filter_call))]
pub struct FilterCallNode {
    pub name: Identifier,
    pub args: Vec<FilterArgNode>,
}

#[derive(FromPest)]
#[pest_ast(rule(Rule::filter_arg))]
pub struct FilterArgNode {
    #[pest_ast(outer(with(span_to_string)))]
    pub value: String,
}

// === Function ===

#[derive(FromPest)]
#[pest_ast(rule(Rule::function_block))]
pub struct FunctionBlock {
    #[pest_ast(outer(with(span_to_source_range)))]
    pub source_range: SourceRange,
    pub name: Identifier,
    pub params: Option<FunctionParams>,
    pub return_type: TypeExpr,
    pub body: SurqlBlock,
    pub attributes: Vec<FunctionAttributeLine>,
}

#[derive(FromPest)]
#[pest_ast(rule(Rule::function_params))]
pub struct FunctionParams {
    pub params: Vec<FunctionParamNode>,
}

#[derive(FromPest)]
#[pest_ast(rule(Rule::function_param))]
pub struct FunctionParamNode {
    pub name: Identifier,
    pub type_expr: TypeExpr,
}

#[derive(FromPest)]
#[pest_ast(rule(Rule::function_attribute_line))]
pub struct FunctionAttributeLine {
    pub attribute: BlockAttribute,
}

// === Identifier ===

#[derive(FromPest)]
#[pest_ast(rule(Rule::identifier))]
pub struct Identifier {
    #[pest_ast(outer(with(span_to_source_range)))]
    pub source_range: SourceRange,
    #[pest_ast(outer(with(span_to_string)))]
    pub value: String,
}

#[derive(FromPest)]
#[pest_ast(rule(Rule::EOI))]
struct Eoi;

// === Conversions to public AST ===

impl Schema {
    pub fn into_ast(self) -> ast::Schema {
        ast::Schema {
            items: self.items.into_iter().map(SchemaItem::into_ast).collect(),
        }
    }
}

impl SourceFile {
    pub fn into_ast(self) -> ast::Schema {
        ast::Schema {
            items: self
                .items
                .into_iter()
                .filter_map(SourceItem::into_ast)
                .collect(),
        }
    }
}

impl SchemaItem {
    fn into_ast(self) -> ast::SchemaItem {
        match self {
            SchemaItem::DocComment(doc_comment) => doc_comment.into_ast(),
            SchemaItem::TableBlock(table) => ast::SchemaItem::TableDecl(table.into_ast()),
            SchemaItem::AnalyzerBlock(analyzer) => {
                ast::SchemaItem::AnalyzerDecl(analyzer.into_ast())
            }
            SchemaItem::FunctionBlock(function) => {
                ast::SchemaItem::FunctionDecl(function.into_ast())
            }
        }
    }
}

impl SourceItem {
    fn into_ast(self) -> Option<ast::SchemaItem> {
        match self {
            SourceItem::DocComment(doc_comment) => Some(doc_comment.into_ast()),
            SourceItem::TableBlock(table) => Some(ast::SchemaItem::TableDecl(table.into_ast())),
            SourceItem::AnalyzerBlock(analyzer) => {
                Some(ast::SchemaItem::AnalyzerDecl(analyzer.into_ast()))
            }
            SourceItem::FunctionBlock(function) => {
                Some(ast::SchemaItem::FunctionDecl(function.into_ast()))
            }
            SourceItem::InvalidLine(_) => None,
        }
    }
}

impl DocComment {
    fn into_ast(self) -> ast::SchemaItem {
        let text = self
            .lines
            .iter()
            .map(|line| line.content.trim_start_matches("///").trim())
            .collect::<Vec<_>>()
            .join("\n");

        ast::SchemaItem::DocComment { text }
    }
}

fn extract_surql_body(source: &str) -> String {
    let body_start = source.find('{').map_or(0, |idx| idx + 1);
    let body_end = source.rfind('}').unwrap_or(source.len());
    source[body_start..body_end].to_string()
}

fn extract_surql_inline_body(source: &str) -> String {
    let body_start = source.find('`').map_or(0, |idx| idx + 1);
    let body_end = source.rfind('`').unwrap_or(source.len());
    source[body_start..body_end].to_string()
}

impl TableBlock {
    fn into_ast(self) -> ast::Table {
        let Identifier {
            value: name,
            source_range: name_range,
        } = self.name;
        let mut fields = Vec::new();
        let mut raw_attributes = Vec::new();
        for member in self.members {
            match member {
                TableMember::FieldLine(line) => fields.push(line.into_ast()),
                TableMember::BlockAttributeLine(line) => {
                    raw_attributes.push(line.attribute.into_attribute())
                }
            }
        }
        ast::Table {
            name,
            source_range: Some(self.source_range),
            name_range: Some(name_range),
            modifier: self.modifier.map(|modifier| modifier.value),
            fields,
            indexes: Vec::new(), // populated by semantic lowering
            raw_attributes,
        }
    }
}

impl FieldLine {
    fn into_ast(self) -> ast::Field {
        self.field.into_ast()
    }
}

impl FieldNode {
    fn into_ast(self) -> ast::Field {
        let Identifier {
            value: name,
            source_range: name_range,
        } = self.name;
        let (ty, optional) = self.type_expr.into_field_type();
        let mut raw_attributes: Vec<_> = self
            .attributes
            .into_iter()
            .map(AttributeNode::into_attribute)
            .collect();
        if let Some(block) = self.attribute_block {
            raw_attributes.extend(
                block
                    .attributes
                    .into_iter()
                    .map(|line| line.attribute.into_attribute()),
            );
        }
        ast::Field {
            name,
            source_range: Some(self.source_range),
            name_range: Some(name_range),
            ty,
            optional,
            flexible: false, // populated by semantic lowering
            raw_attributes,
        }
    }
}

impl TypeNode {
    fn into_ast(self) -> ast::Type {
        match self {
            TypeNode::PrimitiveType(p) => ast::Type::Primitive { name: p.value },
            TypeNode::OptionType(o) => ast::Type::Option {
                inner: Box::new((*o.inner).into_ast()),
            },
            TypeNode::ArrayType(a) => ast::Type::Array {
                inner: Box::new((*a.inner).into_ast()),
                length: a.length.and_then(|l| l.digits.parse().ok()),
            },
            TypeNode::SetType(s) => ast::Type::Set {
                inner: Box::new((*s.inner).into_ast()),
                length: s.length.and_then(|l| l.digits.parse().ok()),
            },
            TypeNode::RecordType(r) => ast::Type::Record {
                table: r.table.map(|i| i.value),
            },
            TypeNode::GeometryType(g) => ast::Type::Geometry {
                features: g.features.into_iter().map(|f| f.value).collect(),
            },
        }
    }
}

impl TypeExpr {
    fn into_type(self) -> ast::Type {
        let optional = self.optional.is_some();
        let ty = self.type_node.into_ast();
        if optional && !matches!(ty, ast::Type::Option { .. }) {
            ast::Type::Option {
                inner: Box::new(ty),
            }
        } else {
            ty
        }
    }

    fn into_field_type(self) -> (ast::Type, bool) {
        let optional_from_marker = self.optional.is_some();
        let raw = self.type_node.into_ast();
        // Normalize: if the parsed top-level type is `option<T>`, lift the
        // inner type and set `optional = true`. So `option<int>` and `int?`
        // produce identical AST. Nested `Type::Option` (inside compound types)
        // is left alone.
        match raw {
            ast::Type::Option { inner } => (*inner, true),
            other => (other, optional_from_marker),
        }
    }
}

impl AttributeNode {
    fn into_attribute(self) -> ast::Attribute {
        ast::Attribute {
            name: self.name.value.strip_prefix('@').unwrap().to_string(),
            args: self.call.map(AttrCall::into_args).unwrap_or_default(),
            source_range: Some(self.source_range),
        }
    }
}

impl BlockAttribute {
    fn into_attribute(self) -> ast::Attribute {
        ast::Attribute {
            name: self.name.value.strip_prefix("@@").unwrap().to_string(),
            args: self.call.map(AttrCall::into_args).unwrap_or_default(),
            source_range: Some(self.source_range),
        }
    }
}

impl AttrCall {
    fn into_args(self) -> Vec<ast::AttributeArg> {
        self.args
            .map(|list| list.args.into_iter().map(AttrArg::into_ast).collect())
            .unwrap_or_default()
    }
}

impl AttrArg {
    fn into_ast(self) -> ast::AttributeArg {
        match self {
            AttrArg::Kv(kv) => kv.into_ast(),
            AttrArg::Value(value) => ast::AttributeArg::Positional {
                value: value.into_ast(),
            },
        }
    }
}

impl AttrKv {
    fn into_ast(self) -> ast::AttributeArg {
        ast::AttributeArg::Keyword {
            name: self.name.value,
            value: self.value.into_ast(),
        }
    }
}

impl AttrValueNode {
    fn into_ast(self) -> ast::AttributeValue {
        match self {
            AttrValueNode::Surql(surql) => ast::AttributeValue::Surql {
                body: extract_surql_body(&surql.source),
                source_range: Some(surql.source_range),
            },
            AttrValueNode::SurqlInline(surql) => ast::AttributeValue::Surql {
                body: extract_surql_inline_body(&surql.source),
                source_range: Some(surql.source_range),
            },
            AttrValueNode::Number(n) => ast::AttributeValue::Number {
                value: n.value.parse().unwrap_or(0.0),
            },
            AttrValueNode::Bool(b) => ast::AttributeValue::Bool {
                value: b.value == "true",
            },
            AttrValueNode::Ident(i) => ast::AttributeValue::Ident { value: i.value },
            AttrValueNode::String(s) => ast::AttributeValue::String {
                // strip surrounding quotes
                value: s
                    .value
                    .strip_prefix('"')
                    .and_then(|s| s.strip_suffix('"'))
                    .unwrap_or(&s.value)
                    .to_string(),
            },
            AttrValueNode::Array(arr) => ast::AttributeValue::Array {
                values: arr
                    .values
                    .into_iter()
                    .map(AttrValueNode::into_ast)
                    .collect(),
            },
            AttrValueNode::Tuple(t) => ast::AttributeValue::Tuple {
                values: t.values.into_iter().map(AttrValueNode::into_ast).collect(),
            },
        }
    }
}

impl AnalyzerBlock {
    fn into_ast(self) -> ast::Analyzer {
        let Identifier {
            value: name,
            source_range: name_range,
        } = self.name;
        let mut tokenizers = Vec::new();
        let mut filters = Vec::new();
        for member in self.members {
            match member {
                AnalyzerMember::ClauseLine(line) => match line.clause {
                    AnalyzerClause::Tokenizers(t) => {
                        tokenizers.extend(t.names.into_iter().map(|i| i.value))
                    }
                    AnalyzerClause::Filters(f) => {
                        filters.extend(f.calls.into_iter().map(FilterCallNode::into_ast))
                    }
                },
            }
        }
        ast::Analyzer {
            name,
            source_range: Some(self.source_range),
            name_range: Some(name_range),
            tokenizers,
            filters,
        }
    }
}

impl FilterCallNode {
    fn into_ast(self) -> ast::FilterCall {
        ast::FilterCall {
            name: self.name.value,
            args: self.args.into_iter().map(|a| a.value).collect(),
        }
    }
}

impl FunctionBlock {
    fn into_ast(self) -> ast::Function {
        let Identifier {
            value: name,
            source_range: name_range,
        } = self.name;
        ast::Function {
            name,
            source_range: Some(self.source_range),
            name_range: Some(name_range),
            params: self
                .params
                .map(|params| {
                    params
                        .params
                        .into_iter()
                        .map(FunctionParamNode::into_ast)
                        .collect()
                })
                .unwrap_or_default(),
            return_type: self.return_type.into_type(),
            body: ast::SurqlBlock {
                body: extract_surql_body(&self.body.source),
            },
            raw_attributes: self
                .attributes
                .into_iter()
                .map(|line| line.attribute.into_attribute())
                .collect(),
        }
    }
}

impl FunctionParamNode {
    fn into_ast(self) -> ast::FunctionParam {
        let Identifier {
            value: name,
            source_range: name_range,
        } = self.name;
        ast::FunctionParam {
            name,
            name_range: Some(name_range),
            ty: self.type_expr.into_type(),
        }
    }
}
