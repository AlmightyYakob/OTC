use swc_ecma_ast::{KeyValueProp, MethodProp, ObjectLit, Prop, PropName};

// WIP: AST to contain vue component
#[derive(Debug)]
pub struct VueComponent {
    pub components: Option<KeyValueProp>,
    pub props: Option<KeyValueProp>,
    pub data: Option<MethodProp>,
    pub created: Option<MethodProp>,
    pub mounted: Option<MethodProp>,
    pub methods: Option<KeyValueProp>,
}
impl Default for VueComponent {
    fn default() -> VueComponent {
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

pub fn create_vue_component(object: ObjectLit) -> VueComponent {
    let mut vue = VueComponent::default();
    for prop in object
        .props
        .into_iter()
        .map(|x| x.prop())
        .filter(|x| x.is_some())
        .map(|x| x.unwrap())
    {
        let key = match *prop {
            Prop::KeyValue(ref x) => Some(&x.key),
            Prop::Method(ref x) => Some(&x.key),
            _ => None,
        };
        if key.is_none() {
            continue;
        }

        // Extract name
        let propname = key.unwrap();
        let keystr = match propname {
            PropName::Ident(ref id) => Some(&id.sym),
            PropName::Str(ref s) => Some(&s.value),
            _ => None,
        };
        if keystr.is_none() {
            continue;
        }

        // Assign
        match keystr.unwrap().to_string().as_str() {
            "components" => vue.components = Some(prop.key_value().unwrap()),
            "props" => vue.props = Some(prop.key_value().unwrap()),
            "data" => vue.data = Some(prop.method().unwrap()),
            "created" => vue.created = Some(prop.method().unwrap()),
            "mounted" => vue.mounted = Some(prop.method().unwrap()),
            "methods" => vue.methods = Some(prop.key_value().unwrap()),
            _ => {}
        }
    }

    return vue;
}
