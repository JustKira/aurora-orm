// Semantic tests assert meaning after the parser has accepted the source.
// Parser tests answer "is the syntax valid?"; these modules answer "does the
// schema make sense, and did raw attributes lower into checked schema data?"

#[path = "semantic/common.rs"]
mod common;

#[path = "semantic/analyzers.rs"]
mod analyzers;
#[path = "semantic/attributes.rs"]
mod attributes;
#[path = "semantic/fulltext.rs"]
mod fulltext;
#[path = "semantic/functions.rs"]
mod functions;
#[path = "semantic/indexes.rs"]
mod indexes;
#[path = "semantic/pipeline.rs"]
mod pipeline;
#[path = "semantic/surql.rs"]
mod surql;
#[path = "semantic/symbols.rs"]
mod symbols;
#[path = "semantic/types.rs"]
mod types;
#[path = "semantic/vector.rs"]
mod vector_indexes;
