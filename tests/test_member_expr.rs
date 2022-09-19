use otc::visitor::Visitor;
use swc_core::testing_transform::test;
use swc_ecma_visit::as_folder;

test!(
    Default::default(),
    |_| as_folder(Visitor::default()),
    test_member_expr,
    r#"export default {
        props: {
            something: {
                type: String,
                required: true,
            },
        },
        data() {
            return {
                x: 1,
            }
        },
        methods: {
            method1(arg) {
                // Normal member expression
                this.x = 2;

                // Function call
                this.something.foo.bar.run();

                // Prop
                console.log(this.something);

                // Emit
                this.$emit('foo');

                // Global prop
                this.$foo
            },
        },
    };"#,
    r#"export default defineComponent({
        props: {
            something: {
                type: String,
                required: true,
            }
        },
        setup (props, ctx) {
            const x = ref(1);

            function method1(arg) {
                x.value = 2;
                props.something.foo.bar.run();
                console.log(props.something);
                ctx.$emit('foo');
                ctx.$root.foo;
            }

            return {
                x,
                method1,
            }
        },
    });"#
);
