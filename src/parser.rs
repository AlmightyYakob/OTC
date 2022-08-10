use rslint_parser::{ast::BracketExpr, parse_text, util, AstNode, SyntaxNodeExt, SyntaxToken};

fn parse_script(script: &String) {
    let parse = parse_text(script, 0);
    // The untyped syntax node of `foo.bar[2]`, the root node is `Script`.
    // let untyped_expr_node = parse.syntax().first_child().unwrap();

    println!("{:?}", parse.syntax().children());

    // // SyntaxNodes can be turned into a nice string representation.
    // println!("{:#?}", untyped_expr_node);

    // // You can then cast syntax nodes into a typed AST node.
    // let typed_ast_node =
    //     BracketExpr::cast(untyped_expr_node.first_child().unwrap().to_owned()).unwrap();

    // // Everything on every ast node is optional because of error recovery.
    // let prop = dbg!(typed_ast_node.prop()).unwrap();

    // // You can then go back to an untyped SyntaxNode and get its range, text, parents, children, etc.
    // assert_eq!(prop.syntax().text(), "2");

    // // Util has a function for yielding all tokens of a node.
    // let tokens = untyped_expr_node.tokens();

    // assert_eq!(&util::concat_tokens(&tokens), "foo. bar[2]");
}
