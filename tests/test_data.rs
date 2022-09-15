use otc::visitor::Visitor;
use swc_core::testing_transform::test;
use swc_ecma_visit::as_folder;

test!(
    Default::default(),
    |_| as_folder(Visitor::default()),
    test_basic_data_to_setup,
    // Input codes
    r#"export default {
        data() {
            return {
                loading: false,
                foo: null,
                count: 0,
                headers: [
                    {
                        text: 'Name',
                        value: 'name',
                    },
                    {
                        text: 'Identifier',
                        value: 'identifier',
                    },
                ],
            };
        },
    };"#,
    // Output codes after transformed with plugin
    r#"export default defineComponent({
        setup (props, ctx) {
            const loading = ref(false);
            const foo = ref(null);
            const count = ref(0);
            const headers = ref([
                {
                    text: 'Name',
                    value: 'name',
                },
                {
                    text: 'Identifier',
                    value: 'identifier',
                },
            ]);

            return {
                loading,
                foo,
                count,
                headers,
            }
        },
    });"#
);
