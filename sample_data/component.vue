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
    inject: ['something'],
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
    computed: {
        bar() {
            return this.foo || 'bar';
        },
    },
    created() {
        this.foo = 1;
    },
    async mounted() {
        this.loading = true;
        this.method1()
        this.count += 1;
        this.loading = false;
    },
    methods: {
        method1() {
            console.log('nothing!')
        },
        async method2() {
            console.log('async!')
        },
    },
};
</script>
