use jj_lib::revset::{parse_query, parse_tree, parse_diff, parse, RevsetParseContext, RevsetAliasesMap, RevsetDiagnostics, RevsetExtensions};
use jj_lib::fileset::FilesetAliasesMap;
use std::collections::HashMap;

#[test]
fn test_query_types() {
    let context = RevsetParseContext {
        aliases_map: &RevsetAliasesMap::default(),
        local_variables: HashMap::new(),
        user_email: "",
        date_pattern_context: chrono::Local::now().into(),
        default_ignored_remote: None,
        fileset_aliases_map: &FilesetAliasesMap::new(),
        use_glob_by_default: true,
        extensions: &RevsetExtensions::default(),
        workspace: None,
    };
    
    // Revsets
    assert!(parse(&mut RevsetDiagnostics::new(), "all()", &context).is_ok());
    let err = parse(&mut RevsetDiagnostics::new(), "tree(all())", &context).unwrap_err();
    assert_eq!(err.kind().to_string(), "Type error: Expected a revset, got a tree");
    
    let err2 = parse(&mut RevsetDiagnostics::new(), "all() & tree(all())", &context).unwrap_err();
    assert_eq!(err2.kind().to_string(), "Type error: Expected a revset, got a tree");

    // Trees
    assert!(parse_tree(&mut RevsetDiagnostics::new(), "tree(all())", &context).is_ok());
    assert!(parse_tree(&mut RevsetDiagnostics::new(), "merged_of(tree(all()), tree(none()))", &context).is_ok());
    assert!(parse_tree(&mut RevsetDiagnostics::new(), "rebase_of(tree(all()), all())", &context).is_ok());
    assert!(parse_tree(&mut RevsetDiagnostics::new(), "revert_of(tree(all()), diff(tree(none()), tree(all())))", &context).is_ok());
    assert!(parse_tree(&mut RevsetDiagnostics::new(), "revert_of(tree(all()), all())", &context).is_ok()); // coercion in diff argument!

    assert!(parse_tree(&mut RevsetDiagnostics::new(), "all()", &context).is_ok()); // Coerced tree!
    let err4 = parse_tree(&mut RevsetDiagnostics::new(), "diff(tree(all()), tree(none()))", &context).unwrap_err();
    assert_eq!(err4.kind().to_string(), "Type error: Expected a tree, got a diff");
    
    // Diffs
    assert!(parse_diff(&mut RevsetDiagnostics::new(), "diff(tree(all()), tree(none()))", &context).is_ok());
    assert!(parse_diff(&mut RevsetDiagnostics::new(), "invert(diff(tree(all()), tree(none())))", &context).is_ok());
    assert!(parse_diff(&mut RevsetDiagnostics::new(), "all()", &context).is_ok()); // Coerced diff!
}
