use anyhow::{anyhow, Result};
use std::path::Path;
use tree_sitter::{Parser, Query, QueryCursor};

pub struct SemanticAnalyzer {
    parser: Parser,
}

impl SemanticAnalyzer {
    pub fn new() -> Self {
        let mut parser = Parser::new();
        // Default to TypeScript for now, ideally we'd switch based on file extension
        parser.set_language(tree_sitter_typescript::language_typescript()).expect("Error loading TypeScript grammar");
        Self { parser }
    }

    pub fn analyze_file(&mut self, path: &Path) -> Result<Vec<String>> {
        let source_code = std::fs::read_to_string(path)?;
        let tree = self.parser.parse(&source_code, None).ok_or_else(|| anyhow!("Failed to parse file"))?;
        let root_node = tree.root_node();

        // Simple query to find function definitions
        let query_string = "(function_declaration name: (identifier) @name) (method_definition name: (property_identifier) @name)";
        let query = Query::new(tree_sitter_typescript::language_typescript(), query_string)?;
        let mut query_cursor = QueryCursor::new();
        
        let mut functions = Vec::new();
        for match_ in query_cursor.matches(&query, root_node, source_code.as_bytes()) {
            for capture in match_.captures {
                let range = capture.node.byte_range();
                let name = &source_code[range];
                functions.push(name.to_string());
            }
        }

        Ok(functions)
    }
}
