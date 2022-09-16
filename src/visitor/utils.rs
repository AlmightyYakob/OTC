use std::{
    collections::{HashMap, HashSet},
    iter::FromIterator,
};

use super::vue::Inject;

use string_cache::Atom;
use swc_ecma_ast::*;

/** Represents some structure that may want to be ordered */
#[derive(Debug)]
pub struct Ordered<T> {
    pub order: usize,
    pub value: T,
}

/** Return the set of injections from an object lit */
pub fn inject_set_from_object_lit(expr: &Box<Expr>) -> Option<HashMap<String, Ordered<Inject>>> {
    let mut values: Vec<Ordered<Inject>> = vec![];

    // Handle array literal of string literal
    if let Expr::Array(arr) = &**expr {
        values = arr
            .elems
            .iter()
            .enumerate()
            .filter_map(|(index, elem)| {
                if let Some(expr_or_spread) = elem {
                    if let Expr::Lit(lit) = &*expr_or_spread.expr {
                        if let Lit::Str(string_lit) = lit {
                            return Some(Ordered {
                                order: index,
                                value: Inject {
                                    name: string_lit.value.to_string(),
                                    from: expr_or_spread.expr.clone(),
                                    default: None,
                                },
                            });
                        }
                    }
                }

                return None;
            })
            .collect();
    }

    // Handle object literal (renaming / defaults)
    if let Expr::Object(obj) = &**expr {
        values = obj
            .props
            .iter()
            .enumerate()
            .filter_map(|(index, elem)| {
                if !elem.is_prop() {
                    return None;
                }

                let prop = elem.as_prop().unwrap();
                if !prop.is_key_value() {
                    return None;
                }

                // Get kv, key must be ident
                let KeyValueProp { key, value } = &*prop.as_key_value().unwrap();
                if !key.is_ident() {
                    return None;
                }

                // Extract injection name
                let name = key.as_ident().unwrap().sym.to_string();

                // If not object lit, just pass along expression
                if !value.is_object() {
                    return Some(Ordered {
                        order: index,
                        value: Inject {
                            name,
                            from: value.clone(),
                            default: None,
                        },
                    });
                }

                let obj = value.as_object().unwrap();

                // Either `from` or `default` must be present as props
                let mut from: Option<Box<Expr>> = None;
                let mut default: Option<Box<Expr>> = None;
                for el in obj.props.iter() {
                    if !el.is_prop() {
                        continue;
                    }

                    let prop = el.as_prop().unwrap();
                    match &**prop {
                        Prop::KeyValue(kv) => {
                            if let PropName::Ident(id) = &kv.key {
                                match id.sym.to_string().as_str() {
                                    "from" => {
                                        from = Some(kv.value.clone());
                                    }
                                    "default" => {
                                        default = Some(kv.value.clone());
                                    }
                                    _ => {}
                                }
                            }
                        }
                        Prop::Shorthand(id) => match id.sym.to_string().as_str() {
                            "from" => {
                                from = Some(Box::new(Expr::Ident(id.clone())));
                            }
                            "default" => {
                                default = Some(Box::new(Expr::Ident(id.clone())));
                            }
                            _ => {}
                        },
                        _ => {}
                    }
                }

                // Return Some if either required field present
                if from.is_some() || default.is_some() {
                    let from = match from {
                        Some(val) => val,
                        None => Box::new(Expr::Lit(Lit::Str(Str {
                            span: Default::default(),
                            raw: None,
                            value: Atom::from(name.clone()),
                        }))),
                    };

                    return Some(Ordered {
                        order: index,
                        value: Inject {
                            name,
                            from,
                            default,
                        },
                    });
                }

                return None;
            })
            .collect();
    }

    // Return if found
    if values.len() > 0 {
        let mut map = HashMap::new();
        for inject in values.into_iter() {
            map.insert(inject.value.name.clone(), inject);
        }

        return Some(map);
    }

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
