use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "aureline.pest"]
pub struct AurelineParser;
