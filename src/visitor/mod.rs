use std::{
    collections::{HashMap, HashSet},
    iter::FromIterator,
};

use string_cache::Atom;
use swc_common::comments::{Comments, SingleThreadedComments};
use swc_common::Span;
use swc_ecma_ast::*;
use swc_ecma_visit::{as_folder, FoldWith, Visit, VisitMut, VisitMutWith, VisitWith};

use self::{utils::Ordered, vue::Inject};

// Modules
pub mod process;
pub mod transform;
pub mod utils;
pub mod vue;
pub mod write;

const SPECIAL_FUNCTIONS: [&str; 1] = ["$emit"];

pub struct Visitor {
    pub options: vue::OptionsComponent,
    pub composition: vue::CompositionComponent,
    // TODO: Set this to false if there is ever some issue parsing vue file,
    // and skip that file if so
    // valid: bool,

    // Track props
    pub props_set: Option<HashSet<String>>,

    // Track injects, preserving definition order
    pub inject_set: Option<HashMap<String, Ordered<Inject>>>,
    pub special_functions: HashSet<String>,
    // Comments
    pub comments: SingleThreadedComments,
}
impl Default for Visitor {
    fn default() -> Visitor {
        Self {
            options: Default::default(),
            composition: Default::default(),
            // valid: true,
            props_set: Default::default(),
            inject_set: Default::default(),
            special_functions: HashSet::from_iter(SPECIAL_FUNCTIONS.map(|s| s.to_string())),
            comments: Default::default(),
        }
    }
}

// This is used for analysis before modification
impl Visit for Visitor {
    // fn visit_span(&mut self, span: &Span) {
    //     dbg!();
    // }

    fn visit_module_decl(&mut self, decl: &ModuleDecl) {
        decl.visit_children_with(self);

        if let Some(expr) = decl.as_export_default_expr() {
            if let Some(obj) = expr.expr.as_object() {
                // dbg!(self.comments.get_leading(obj.span.lo));
                self.preprocess_default_export(obj);
            }
        }
    }
}

// This is used for the AST modification
impl VisitMut for Visitor {
    // Since functions aren't defined as refs, they must be handled here first
    fn visit_mut_call_expr(&mut self, call_expr: &mut CallExpr) {
        if let Callee::Expr(e) = &mut call_expr.callee {
            if let Expr::Member(member_expr) = &mut **e {
                if let (Expr::This(_), MemberProp::Ident(id)) =
                    (&*member_expr.obj, &member_expr.prop)
                {
                    // Simply replace this.method() with method(),
                    // excluding any special functions
                    if !self.special_functions.contains(&id.sym.to_string()) {
                        call_expr.callee = Callee::Expr(Box::new(Expr::Ident(id.clone())));
                    }
                }
            }
        }

        // Visit children after top level processing, since calls are
        // higher up in the AST, and we're removing the `this` expression
        call_expr.visit_mut_children_with(self);
    }

    // This will convert all uses of `this` to the corresponding refs
    fn visit_mut_member_expr(&mut self, member_expr: &mut MemberExpr) {
        // Visit children before top level processing
        member_expr.visit_mut_children_with(self);

        // Handle injects, since they convert member expressions to idents
        // If a `this` expression is found here, it means it's an inject, since
        // otherwise it would have been transformed deeper within the tree before reaching here
        if let (Expr::Member(nested_member_expr), MemberProp::Ident(_)) =
            (&*member_expr.obj, &member_expr.prop)
        {
            if let (Expr::This(_), MemberProp::Ident(nested_id)) =
                (&*nested_member_expr.obj, &nested_member_expr.prop)
            {
                member_expr.obj = Box::new(Expr::Ident(Ident {
                    optional: false,
                    span: Default::default(),
                    sym: nested_id.sym.clone(),
                }))
            }
        }

        // Handle most nested case
        if let (Expr::This(_), MemberProp::Ident(id)) = (&*member_expr.obj, &mut member_expr.prop) {
            // Check if id is an inject
            if let Some(injects) = &self.inject_set {
                if injects.contains_key(&id.sym.to_string()) {
                    // Don't do anything, this is handled higher up the tree
                    return;
                }
            }

            // Handle props
            if let Some(props) = &self.props_set {
                // Replace `this` with `props` if its in the props set
                if props.contains(&id.sym.to_string()) {
                    member_expr.obj = Box::new(Expr::Ident(Ident {
                        optional: false,
                        span: Default::default(),
                        sym: Atom::from("props"),
                    }));

                    // Exit early
                    return;
                }
            }

            // Handle $emit
            let value_string = id.sym.to_string();
            if value_string.as_str() == "$emit" {
                member_expr.obj = Box::new(Expr::Ident(Ident {
                    optional: false,
                    span: Default::default(),
                    sym: Atom::from("ctx"),
                }));

                // Exit early
                return;
            }

            // Handle arbitrary global props
            if value_string.as_str().chars().next().unwrap() == '$' {
                // Convert this.$foo to ctx.$root.foo
                member_expr.obj = Box::new(Expr::Member(MemberExpr {
                    span: Default::default(),
                    obj: Box::new(Expr::Ident(Ident {
                        optional: false,
                        span: Default::default(),
                        sym: Atom::from("ctx"),
                    })),
                    prop: MemberProp::Ident(Ident {
                        optional: false,
                        span: Default::default(),
                        sym: Atom::from("$root"),
                    }),
                }));

                // Remove `$` from ident
                id.sym = Atom::from(&value_string[1..]);

                // Exit early
                return;
            }

            // Default case, treat as ref
            member_expr.obj = Box::new(Expr::Ident(id.clone()));
            member_expr.prop = MemberProp::Ident(Ident {
                optional: false,
                span: Default::default(),
                sym: Atom::from("value"),
            });
        }
    }

    fn visit_mut_module(&mut self, module: &mut Module) {
        // Preprocess before mutating module
        module.visit_with(self);

        // Visit children before top level processing
        module.visit_mut_children_with(self);

        // Find default export
        let res = module.body.iter().enumerate().find_map(|(index, item)| {
            if !item.is_module_decl() {
                return None;
            }

            let decl = item.as_module_decl().unwrap();
            if !decl.is_export_default_expr() {
                return None;
            }

            let expr = decl.as_export_default_expr().unwrap();
            if !expr.expr.is_object() {
                return None;
            }

            return Some((index, expr.expr.as_object().unwrap().clone()));
        });

        // Exit if not found
        if res.is_none() {
            return;
        }

        // Process default export props
        let (default_export_index, default_export) = res.unwrap();
        self.process_default_export(&default_export);

        // Run all transformations between options and composition API
        // dbg!(&self.composition);
        self.transform_component();

        // Convert
        module.body[default_export_index] = ModuleItem::ModuleDecl(ModuleDecl::ExportDefaultExpr(
            write::write_composition_component(&self.composition),
        ))
    }
}

pub fn visit_module(module: Module) -> Module {
    // dbg!(&module);
    module.fold_with(&mut as_folder(Visitor::default()))
}
