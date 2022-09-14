use super::utils;
use super::vue::WatchDecl;
use super::Visitor;

use swc_ecma_ast::*;

impl Visitor {
    pub fn preprocess_default_export(&mut self, object: &ObjectLit) {
        // Build set of prop IDs
        for x in object.props.iter() {
            if let Some(prop) = x.as_prop() {
                if let Prop::KeyValue(kv) = &**prop {
                    if let Some(ident) = kv.key.as_ident() {
                        match ident.sym.to_string().as_str() {
                            "props" => {
                                self.props_set = utils::prop_set_from_object_lit(&kv.value);
                            }
                            "inject" => {
                                self.inject_set = utils::inject_set_from_object_lit(&kv.value);
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
    }

    pub fn process_method_prop(&mut self, method_prop: &MethodProp) {
        if let Some(ident) = method_prop.key.as_ident() {
            match ident.sym.to_string().as_str() {
                "data" => {
                    self.options.data = Some(method_prop.function.clone());
                }
                "created" => {
                    self.options.created = Some(method_prop.function.clone());
                }
                "mounted" => {
                    self.options.mounted = Some(method_prop.function.clone());
                }
                _ => {}
            }
        }
    }

    pub fn process_computed(&mut self, obj: &ObjectLit) {
        let mut computed_decls: Vec<FnDecl> = vec![];
        for prop in obj.props.iter() {
            if let PropOrSpread::Prop(boxed_expr) = prop {
                if let Prop::Method(method_expr) = &**boxed_expr {
                    if let PropName::Ident(ident) = &method_expr.key {
                        computed_decls.push(FnDecl {
                            ident: ident.clone(),
                            declare: false,
                            function: method_expr.function.clone(),
                        });
                    }
                }
            }
        }

        // Add to component
        if computed_decls.len() > 0 {
            self.options.computed = Some(computed_decls);
        }
    }

    pub fn process_watch(&mut self, obj: &ObjectLit) {
        let mut watch_decls: Vec<WatchDecl> = vec![];
        for prop in obj.props.iter() {
            if let PropOrSpread::Prop(boxed_expr) = prop {
                // Method case
                if let Prop::Method(method_expr) = &**boxed_expr {
                    if let PropName::Ident(ident) = &method_expr.key {
                        watch_decls.push(WatchDecl {
                            ident: ident.clone(),
                            function: method_expr.function.clone(),
                            deep: None,
                            immediate: None,
                        });
                    }
                }

                // Complex case
                if let Prop::KeyValue(kv_prop) = &**boxed_expr {
                    if let Expr::Object(obj) = &*kv_prop.value {
                        if let PropName::Ident(ident) = &kv_prop.key {
                            let mut function: Option<Function> = None;
                            let mut immediate: Option<Box<Expr>> = None;
                            let mut deep: Option<Box<Expr>> = None;

                            // Find handler, deep, and immediate
                            for item in &obj.props {
                                if !item.is_prop() {
                                    continue;
                                }
                                let prop = item.as_prop().unwrap();

                                match &**prop {
                                    // Check for handler method
                                    Prop::Method(method) => {
                                        if let PropName::Ident(inner_ident) = &method.key {
                                            if inner_ident.sym.to_string().as_str() == "handler" {
                                                function = Some(method.function.clone());
                                            }
                                        }
                                    }
                                    // Check for deep/immediate
                                    Prop::KeyValue(inner_kv) => {
                                        if let PropName::Ident(inner_ident) = &inner_kv.key {
                                            if inner_ident.sym.to_string().as_str() == "deep" {
                                                deep = Some(inner_kv.value.clone());
                                            }

                                            if inner_ident.sym.to_string().as_str() == "immediate" {
                                                immediate = Some(inner_kv.value.clone());
                                            }
                                        }
                                    }
                                    _ => {}
                                }
                            }

                            // Check
                            if let Some(func) = function {
                                watch_decls.push(WatchDecl {
                                    ident: ident.clone(),
                                    function: func,
                                    immediate,
                                    deep,
                                })
                            }
                        }
                    }
                }
            }
        }

        // Add to component
        if watch_decls.len() > 0 {
            self.options.watch = Some(watch_decls);
        }
    }

    pub fn process_methods(&mut self, obj: &ObjectLit) {
        let mut methods: Vec<FnDecl> = vec![];
        for prop in obj.props.iter() {
            if let PropOrSpread::Prop(boxed_expr) = prop {
                if let Prop::Method(method_expr) = &**boxed_expr {
                    if let PropName::Ident(ident) = &method_expr.key {
                        methods.push(FnDecl {
                            ident: ident.clone(),
                            declare: false,
                            function: method_expr.function.clone(),
                        });
                    }
                }
            }
        }

        // Add to component
        if methods.len() > 0 {
            self.options.methods = Some(methods);
        }
    }

    pub fn process_key_value(&mut self, kv: &KeyValueProp) {
        if let Some(ident) = kv.key.as_ident() {
            match ident.sym.to_string().as_str() {
                "components" => {
                    self.options.components = Some(kv.value.clone());
                }
                "inject" => {
                    self.options.inject = Some(kv.value.clone());
                }
                "props" => {
                    self.options.props = Some(kv.value.clone());
                }
                "computed" => {
                    if let Expr::Object(obj) = &*kv.value {
                        self.process_computed(obj);
                    }
                }
                "watch" => {
                    if let Expr::Object(obj) = &*kv.value {
                        self.process_watch(obj);
                    }
                }
                "methods" => {
                    if let Expr::Object(obj) = &*kv.value {
                        self.process_methods(obj);
                    }
                }
                _ => {}
            }
        }
    }

    pub fn process_default_export(&mut self, object: &ObjectLit) {
        for prop in object.props.iter() {
            if !prop.is_prop() {
                continue;
            }

            match &**prop.as_prop().unwrap() {
                Prop::Method(method_prop) => {
                    self.process_method_prop(method_prop);
                }
                Prop::KeyValue(kv) => {
                    self.process_key_value(kv);
                }
                _ => {}
            }
        }
    }
}
