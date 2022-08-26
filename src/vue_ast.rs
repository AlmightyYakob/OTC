use swc_ecma_ast::{KeyValueProp, MethodProp};

// WIP: AST to contain vue component
#[derive(Debug)]
pub struct VueOptionsComponent {
    pub components: Option<KeyValueProp>,
    pub props: Option<KeyValueProp>,
    pub data: Option<MethodProp>,
    pub created: Option<MethodProp>,
    pub mounted: Option<MethodProp>,
    pub methods: Option<KeyValueProp>,
}
impl Default for VueOptionsComponent {
    fn default() -> VueOptionsComponent {
        Self {
            components: None,
            props: None,
            data: None,
            created: None,
            mounted: None,
            methods: None,
        }
    }
}

// WIP: AST to contain vue component
#[derive(Debug)]
pub struct VueCompositionComponent {
    pub props: Option<KeyValueProp>,
    pub setup: Option<MethodProp>,
}
impl Default for VueCompositionComponent {
    fn default() -> VueCompositionComponent {
        Self {
            props: None,
            setup: None,
        }
    }
}
