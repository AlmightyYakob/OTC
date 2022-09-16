use otc::visitor::Visitor;
use swc_core::testing_transform::test;
use swc_ecma_visit::as_folder;

test!(
    Default::default(),
    |_| as_folder(Visitor::default()),
    test_inject,
    // Input codes
    r#"export default {
        inject: {
            something: 'something',
            something2: {
                from: 'something2',
                default: () => 3,
            },
            otherInject: {
                default: 'some default',
            },
            noDefaultInject: {
                from: 'something3'
            },
        },
    };"#,
    // Output codes after transformed with plugin
    r#"export default defineComponent({
        setup (props, ctx) {
            const something = inject('something');
            const something2 = inject('something2', ()=>3);
            const otherInject = inject("otherInject", 'some default');
            const noDefaultInject = inject('something3');

            return {
                something,
                something2,
                otherInject,
                noDefaultInject,
            }
        },
    });"#
);
