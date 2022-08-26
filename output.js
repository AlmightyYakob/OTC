import ComponentOne from '@/components/ComponentOne.vue';
import ComponentTwo from '@/components/ComponentTwo.vue';
export default {
    components: {
        ComponentOne,
        ComponentTwo
    },
    inject: [
        'something'
    ],
    props: {
        id: {
            type: String,
            required: true
        }
    },
    setup () {
        const loading = ref(false);
        const foo = ref(null);
        const count = ref(0);
        const headers = ref([
            {
                text: 'Name',
                value: 'name'
            },
            {
                text: 'Identifier',
                value: 'identifier'
            }, 
        ]);
        return {
            loading,
            foo,
            count,
            headers
        };
    },
    created () {
        this.foo = 1;
    },
    async mounted () {
        this.loading = true;
        this.something();
        this.count += 1;
        this.loading = false;
    },
    methods: {
        method1 () {
            console.log('nothing!');
        },
        async method2 () {
            console.log('async!');
        }
    }
};
