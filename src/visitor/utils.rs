use std::{
    collections::{HashMap, HashSet},
    iter::FromIterator,
};

use swc_ecma_ast::{Expr, Lit, PropName, Str};

pub fn inject_set_from_object_lit(expr: &Box<Expr>) -> Option<HashMap<String, Str>> {
    // Handle array literal of string literal
    if let Expr::Array(arr) = &**expr {
        let strings: Vec<(String, Str)> = arr
            .elems
            .iter()
            .filter_map(|elem| {
                if let Some(expr_or_spread) = elem {
                    if let Expr::Lit(lit) = &*expr_or_spread.expr {
                        if let Lit::Str(string_lit) = lit {
                            return Some((string_lit.value.to_string(), string_lit.clone()));
                        }
                    }
                }

                return None;
            })
            .collect();

        // Return if found
        if strings.len() > 0 {
            return Some(HashMap::from_iter(strings));
        }
    }

    // TODO: Handle aliasing with object literal
    // TODO: Handle default values

    return None;
}

pub fn prop_set_from_object_lit(expr: &Box<Expr>) -> Option<HashSet<String>> {
    let mut set: Option<HashSet<String>> = None;

    // Handle arrays
    if let Expr::Array(arr) = &**expr {
        let items: Vec<String> = arr
            .elems
            .iter()
            .filter_map(|item| {
                if item.is_none() {
                    return None;
                }

                let expr_or_spread = item.as_ref().unwrap();
                if !expr_or_spread.expr.is_lit() {
                    return None;
                }

                let lit = expr_or_spread.expr.as_lit().unwrap();
                if let Lit::Str(s) = lit {
                    return Some(s.value.to_string());
                }

                return None;
            })
            .collect();

        // Finally set if necessary
        if items.len() > 0 {
            set = Some(HashSet::from_iter(items));
        }
    }

    // Handle Objects
    if let Expr::Object(obj) = &**expr {
        let items: Vec<String> = obj
            .props
            .iter()
            .filter_map(|item| {
                if !item.is_prop() {
                    return None;
                }

                let prop = item.as_prop().unwrap();
                if !prop.is_key_value() {
                    return None;
                }

                let kv = prop.as_key_value().unwrap();
                return match &kv.key {
                    PropName::Ident(id) => Some(id.sym.to_string()),
                    PropName::Str(s) => Some(s.value.to_string()),
                    _ => None,
                };
            })
            .collect();

        // Finally set if necessary
        if items.len() > 0 {
            set = Some(HashSet::from_iter(items));
        }
    }

    return set;
}
