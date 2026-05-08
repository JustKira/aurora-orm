use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "aurora.pest"]
pub struct AuroraParser;
