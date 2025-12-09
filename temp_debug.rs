use coding_guide_helper::lexer::Lexer;
use coding_guide_helper::parser::Parser;
use coding_guide_helper::ast::Item;

fn main() {
    let input = "#ifdef WINDOWS\nint os = 1;\n#elif defined(LINUX)\nint os = 2;\n#else\nint os = 0;\n#endif\n";
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    let tu = parser.parse();
    
    println!("Items count: {}", tu.items.len());
    if let Some(Item::ConditionalBlock { directive_type, condition, items, .. }) = tu.items.get(0) {
        println!("First item: {} with condition: '{}'", directive_type, condition);
        println!("  Inner items: {}", items.len());
        if items.len() > 1 {
            if let Item::ConditionalBlock { directive_type, condition, .. } = &items[1] {
                println!("  Second inner: {} with condition: '{}'", directive_type, condition);
            }
        }
    }
}
