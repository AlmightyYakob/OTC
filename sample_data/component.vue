<template>
    <div />
</template>

<script>
// Not real
import ComponentOne from '@/components/ComponentOne.vue';
import ComponentTwo from '@/components/ComponentTwo.vue';

export default {
    components: {
        ComponentOne,
        ComponentTwo,
    },
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
    props: {
        id: {
            type: String,
            required: true,
        },
    },
    data() {
        return {
            // loading prop
            loading: false,
            foo: null,
            count: 0,
            // headers
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
    watch: {
        loading(val, oldVal) {
            if (val === true) {
                console.log('now loading!');
                this.foo = 2;
            }
        },
        headers: {
            handler(val) {
                console.log('headers changed', val);
            },
            immediate: true,
            deep: true,
        },
    },
    computed: {
        bar() {
            return this.foo || this.id;
        },
        baz() {
            if (this.loading) {
                return 0;
            }
            return this.count + 1;
        }
    },
    created() {
        this.foo = 1;
        this.$emit('emission');
    },
    async mounted() {
        this.loading = true;
        this.method1(this.count)
        this.count += 1;
        this.loading = false;
    },
    methods: {
        method1(arg) {
            console.log(arg)
        },
        async method2() {
            console.log('async!')
            this.something.foo.bar.run();
        },
    },
};
</script>
